use core::{ops::Div, time::Duration};

use aarch64_cpu::{asm, registers::CNTPCT_EL0};
use tock_registers::interfaces::Readable;

const NANOSEC_PER_SEC: u64 = 1_000_000_000;

#[no_mangle]
static ARCH_TIMER_COUNTER_FREQUENCY: u32 = 0;

fn get_arch_timer_counter_freq() -> u32 {
    unsafe { core::ptr::read_volatile(&ARCH_TIMER_COUNTER_FREQUENCY) }
}

#[derive(PartialOrd, PartialEq, Debug)]
pub struct Counter(u64);

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
