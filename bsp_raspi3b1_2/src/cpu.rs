use core::cell::UnsafeCell;

use aarch64_cpu::asm::{self, barrier};
use aarch64_cpu::registers::VBAR_EL2;
use tock_registers::interfaces::Writeable;

use crate::errors::Errcode;

pub mod exception;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
        asm::wfi();
    }
}

pub(crate) fn init() -> Result<(), Errcode> {
    // Init exceptions
    extern "Rust" {
        static __exception_vector_start: UnsafeCell<()>;
    }
    unsafe {
        VBAR_EL2.set(__exception_vector_start.get() as u64);
    }
    barrier::isb(barrier::SY);

    // Init Irq
    Ok(())
}
