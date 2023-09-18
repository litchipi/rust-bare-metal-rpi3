// TODO   Make timer work
//    - Make RwLock
//    - Set timer resolution
//    - Make Vec
//    - Register new tries to find a free spot before creating a new one

use aarch64_cpu::asm;

type Vec<T> = [T; 5];
type RwLock<T> = crate::sync::RwLock<T>;

pub const TIMER_RESOLUTION_US: u64 = 30;
pub const MAX_TIMERS_COUNT: usize = 10;

pub static TIMER: TimerDriver = TimerDriver::init();

// Do not use this struct for sampling or screen rendering, but everything else is fine
pub struct TimerDriver {
    // registered_timers: RwLock<Vec<u64>>,
    // free: RwLock<Vec<bool>>,
}

impl TimerDriver {
    const fn init() -> TimerDriver {
        TimerDriver {
            // registered_timers: RwLock::new(Vec::new()),
            // free: RwLock::new(Vec::new()),
        }
    }

    // Called inside the Timer IRQ handler
    pub(crate) fn tick(&self) {
        // for (idx, is_free) in self.free.read().iter() {
        //    if !is_free {
        //         let count = self.registered_timers.write().get_mut(&idx);
        //         *count = count.saturating_sub(TIMER_RESOLUTION_US);
        //     }
        // }
        todo!();
    }

    pub fn free(&self, idx: usize) {
        // if self.free.read().get(&idx) {
        //     panic!("Double free on timer {idx}");
        // }
        // *self.free.write().get_mut(&idx) = true;
        todo!();
    }

    // Register new timer to follow, return the number
    pub fn register_new(&self, time_us: u64) -> usize {
        // let idx = self.registered_timers.read().len();
        // assert!((idx + 1) < MAX_TIMERS_COUNT, "Max timers count reached");
        // self.registered_timers.write().push(time_us);
        // self.free.write().push(false);
        // idx
        todo!();
    }

    pub fn set(&self, timer_nb: usize, time_us: u64) {
        // *self.free.write().get_mut(&timer_nb) = false;
        // match self.registered_timers.write().get_mut(&timer_nb) {
        //   Some(timer) => *timer = time_us,
        //   None => panic!("Attempt to set timer {timer_nb}, but it doesn't exist"),
        // }
        todo!();
    }

    // Get remaining time to wait (in microseconds)
    pub fn get(&self, timer_nb: usize) -> u64 {
        // if let Some(rest) = self.registered_timers.read().get(timer_nb) {
        //   rest
        // } else {
        //   panic!("Unable to get timer {timer_nb}: Not there");
        // }
        todo!();
    }

    pub fn wait(&self, time_us: u64) {
        let nb = self.register_new(time_us);
        while self.get(nb) > 0 {
            asm::wfi();
        }
    }
}
