use core::{ops::Div, time::Duration};

use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{CNTP_CTL_EL0, CNTP_CVAL_EL0};
use aarch64_cpu::{asm, registers::CNTPCT_EL0};
use alloc::boxed::Box;
use alloc::vec::Vec;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::irq::{IrqHandler, IrqNumber, IRQ_MANAGER};
use crate::println;
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

        let freq: u128 = u32::from(get_arch_timer_counter_freq()) as u128;
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

const TIMER_IRQ: IrqNumber = IrqNumber::Local(1);

/// Program a timer IRQ to be fired after `delay` has passed.
fn set_timeout_irq(due_time: Duration) {
    println!("Set timeout irq to {due_time:?}");
    let counter_value_target: Counter = match due_time.try_into() {
        Err(msg) => panic!("Error setting timeout: {msg}"),
        Ok(val) => val,
    };

    // Set the compare value register.
    CNTP_CVAL_EL0.set(counter_value_target.0);

    // Kick off the timer.
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
}

/// Conclude a pending timeout IRQ.
fn conclude_timeout_irq() {
    // Disable counting. De-asserts the IRQ.
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::CLEAR);
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
        println!("Register timer IRQ");
        IRQ_MANAGER.register(IrqNumber::Local(1), self);
        IRQ_MANAGER.enable(IrqNumber::Local(1));
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
