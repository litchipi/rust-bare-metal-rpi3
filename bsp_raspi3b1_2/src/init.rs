use crate::errors::Errcode;

pub static mut INIT_DONE: bool = false;

pub fn init_bsp() {
    assert!(unsafe { !INIT_DONE });
    crate::memory::init();
    crate::cpu::init();
    crate::drivers::TIMERS.register_timer();
    //    Init allocator
    //    Init GPU
}

pub fn finish_init() {
    // TODO Start other cores on the scheduler task wait
    unsafe {
        INIT_DONE = true;
    }
}
