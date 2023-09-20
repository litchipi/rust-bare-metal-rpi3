use core::{ops::Div, time::Duration};

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{CNTP_CTL_EL0, CNTP_CVAL_EL0};
use aarch64_cpu::{asm, registers::CNTPCT_EL0};
use alloc::boxed::Box;
use alloc::vec::Vec;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};
use tock_registers::registers::ReadWrite;
use tock_registers::{register_bitfields, register_structs};

use crate::irq::{IrqHandler, IrqNumber, IRQ_MANAGER};
use crate::memory::{MMIODerefWrapper, SYSTIMER_BASE};
use crate::{println, dbg};
use crate::sync::RwLock;

const NANOSEC_PER_SEC: u64 = 1_000_000_000;

#[no_mangle]
static ARCH_TIMER_COUNTER_FREQUENCY: u32 = 0;

fn get_arch_timer_counter_freq() -> u32 {
    unsafe { core::ptr::read_volatile(&ARCH_TIMER_COUNTER_FREQUENCY) }
}

#[derive(PartialOrd, PartialEq, Debug)]
pub struct Counter(pub u64);

impl core::ops::Add for Counter {
    type Output = Counter;

    fn add(self, rhs: Self) -> Self::Output {
        Counter(self.0.add(rhs.0))
    }
}

impl From<Counter> for Duration {
    fn from(value: Counter) -> Self {
        if value.0 == 0 {
            return Duration::ZERO;
        }
        let freq = get_arch_timer_counter_freq() as u64;
        let secs = value.0.div(freq);
        let sub_secs = value.0 % freq;
        let nanos = sub_secs.saturating_mul(NANOSEC_PER_SEC).div(freq) as u32;
        Duration::new(secs, nanos)
    }
}

impl TryFrom<Duration> for Counter {
    type Error = &'static str;

    fn try_from(duration: Duration) -> Result<Self, Self::Error> {
        if duration < Duration::from(Counter(1)) {
            return Ok(Counter(0));
        }

        if duration > Duration::MAX {
            return Err("Conversion error. Duration too big");
        }

        let freq = u128::from(get_arch_timer_counter_freq());
        let duration: u128 = duration.as_nanos();
        let counter_value = duration.saturating_mul(freq).div(NANOSEC_PER_SEC as u128);

        Ok(Counter(counter_value as u64))
    }
}

#[inline(always)]
fn read_cntpct() -> Counter {
    // Prevent that the counter is read ahead of time due to out-of-order execution.
    asm::barrier::isb(asm::barrier::SY);
    Counter(CNTPCT_EL0.get())
}

/// Spin for a given duration.
pub fn spin_for(duration: Duration) {
    let curr_counter_value = read_cntpct();

    let counter_value_delta: Counter = match duration.try_into() {
        Ok(val) => val,
        Err(msg) => {
            // warn!("spin_for: {}. Skipping", msg);
            return;
        }
    };
    let counter_value_target = curr_counter_value + counter_value_delta;

    // Busy wait.
    // Read CNTPCT_EL0 directly to avoid the ISB that is part of [`read_cntpct`].
    while Counter(CNTPCT_EL0.get()) < counter_value_target {}
}

const TIMER_IRQ: IrqNumber = IrqNumber::Basic(0);

/// Program a timer IRQ to be fired after `delay` has passed.
fn set_timeout_irq(due_time: Duration) {
    let reg : MMIODerefWrapper<SysTimerRegister> = MMIODerefWrapper::new(SYSTIMER_BASE);
    println!("Set timeout irq to {due_time:?}");
    let counter_value_target: Counter = match due_time.try_into() {
        Err(msg) => panic!("Error setting timeout: {msg}"),
        Ok(val) => val,
    };

    // println!("Compare counter to {:?}", counter_value_target);
    // println!("Current counter value: {:?}", read_cntpct());
    // spin_for(Duration::from_secs(2));
    // println!("Compare counter to {:?}", counter_value_target);
    // println!("Current counter value: {:?}", read_cntpct());
    CNTP_CVAL_EL0.set(counter_value_target.0);
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
    // reg.C1.set(2_000_000); //counter_value_target.0 as u32);
    // reg.CS.write(CS::M1::CLEAR);
    // reg.CS.write(CS::M1::SET);
}

/// Conclude a pending timeout IRQ.
fn conclude_timeout_irq() {
    let reg : MMIODerefWrapper<SysTimerRegister> = MMIODerefWrapper::new(SYSTIMER_BASE);
    reg.C1.set(0);
    reg.CS.write(CS::M1::CLEAR);
}

pub static TIMERS: TimeManager = TimeManager::init();
pub type TimeoutCallback = Box<dyn Fn() + Send>;

pub struct TimeManager {
    queue: RwLock<Vec<Timeout>>,
}

impl IrqHandler for TimeManager {
    fn handle(&self) -> Result<(), &'static str> {
        conclude_timeout_irq();
        if self.queue.read(|q| q.is_empty()) {
            return Ok(());
        }

        let next_due = self.peek_next_due_time().unwrap();
        let uptime = self.uptime();
        if next_due <= uptime {
            let callbacks = self.queue.write(|queue| {
                queue
                    .iter_mut()
                    .filter(|t| t.due_time <= uptime)
                    .map(|t| &t.callback)
                    .collect::<Vec<&TimeoutCallback>>()
            });

            callbacks.iter().for_each(|callback| callback());

            self.queue.write(|queue| {
                queue.iter_mut().for_each(|t| t.refresh());
                queue.retain(|t| t.due_time > uptime)
            });
        }

        let next_due = self.peek_next_due_time().unwrap();
        if next_due <= self.uptime() {
            self.handle(); // Directly handle it
        } else {
            set_timeout_irq(next_due);
        }

        Ok(())
    }
}

impl TimeManager {
    const fn init() -> TimeManager {
        TimeManager {
            queue: RwLock::new(Vec::new()),
        }
    }

    pub fn register_timer(&'static self) {
        dbg!("DBG    Register timer IRQ\n");
        IRQ_MANAGER.register(TIMER_IRQ, self);
        IRQ_MANAGER.enable(TIMER_IRQ);
    }

    pub fn set_timeout(
        &self,
        delay: Duration,
        period: Option<Duration>,
        callback: TimeoutCallback,
    ) {
        let timeout = Timeout {
            due_time: self.uptime() + delay,
            period,
            callback,
        };
        self.queue.write(|q| q.push(timeout));
        set_timeout_irq(self.peek_next_due_time().unwrap());
    }

    pub fn uptime(&self) -> Duration {
        // Prevent that the counter is read ahead of time due to out-of-order execution.
        barrier::isb(barrier::SY);
        let t = Counter(CNTPCT_EL0.get()).into();
        println!("Uptime: {t:?}");
        t
    }

    fn peek_next_due_time(&self) -> Option<Duration> {
        self.queue.read(|q| q.iter().map(|t| t.due_time).min())
    }
}

pub struct Timeout {
    due_time: Duration,
    period: Option<Duration>,
    callback: TimeoutCallback,
}

impl Timeout {
    pub fn refresh(&mut self) {
        if let Some(delay) = self.period {
            self.due_time += delay;
        }
    }
}

register_bitfields! {
    u32,
    CS [
        M0 OFFSET(0) NUMBITS(1) [],
        M1 OFFSET(1) NUMBITS(1) [],
        M2 OFFSET(2) NUMBITS(1) [],
        M3 OFFSET(3) NUMBITS(1) [],
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    SysTimerRegister {
        (0x00 => CS: ReadWrite<u32, CS::Register>),
        (0x04 => CLO: ReadWrite<u32>),
        (0x08 => CHI: ReadWrite<u32>),
        (0x0C => C0: ReadWrite<u32>),
        (0x10 => C1: ReadWrite<u32>),
        (0x14 => C2: ReadWrite<u32>),
        (0x18 => C3: ReadWrite<u32>),
        (0x1C => @END),
    }
}
