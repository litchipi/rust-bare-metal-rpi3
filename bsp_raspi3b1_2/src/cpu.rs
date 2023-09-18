use core::cell::UnsafeCell;

use aarch64_cpu::asm::{self, barrier};
use aarch64_cpu::registers::VBAR_EL2;
use tock_registers::interfaces::Writeable;

pub mod exception;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
        asm::wfi();
    }
}

pub(crate) unsafe fn init_cpu() {
    // Init exceptions
    extern "Rust" {
        static __exception_vector_start: UnsafeCell<()>;
    }
    VBAR_EL2.set(__exception_vector_start.get() as u64);
    barrier::isb(barrier::SY);

    // Init Irq
}
