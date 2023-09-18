use core::cell::UnsafeCell;

use aarch64_cpu::asm::{self, barrier};
use aarch64_cpu::registers::{ESR_EL1, SPSR_EL1, VBAR_EL2};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::InMemoryRegister;

#[inline(always)]
pub fn wait_forever() -> ! {
    loop {
        asm::wfe();
        asm::wfi();
    }
}

struct SpsrEL1(InMemoryRegister<u64, SPSR_EL1::Register>);
struct EsrEL1(InMemoryRegister<u64, ESR_EL1::Register>);

impl EsrEL1 {
    #[inline(always)]
    fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
        self.0.read_as_enum(ESR_EL1::EC)
    }
}

/// The exception context as it is stored on the stack on exception entry.
#[repr(C)]
struct ExceptionContext {
    /// General Purpose Registers.
    gpr: [u64; 30],

    /// The link register, aka x30.
    lr: u64,

    /// Exception link register. The program counter at the time the exception happened.
    elr_el1: u64,

    /// Saved program status.
    spsr_el1: SpsrEL1,

    // Exception syndrome register.
    esr_el1: EsrEL1,
}

#[no_mangle]
extern "C" fn current_el0_synchronous(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_irq(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_serror(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL1 is not supported.")
}

#[no_mangle]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    panic!("current elx irq");
}

#[no_mangle]
extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    panic!("current elx synchronous");
}

#[no_mangle]
extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    panic!("current elx serror");
}

#[no_mangle]
extern "C" fn lower_aarch64_synchronous(e: &mut ExceptionContext) {
    panic!("lower aarch64 synchronous");
}

#[no_mangle]
extern "C" fn lower_aarch64_irq(e: &mut ExceptionContext) {
    panic!("lower aarch64 irq");
}

#[no_mangle]
extern "C" fn lower_aarch64_serror(e: &mut ExceptionContext) {
    panic!("lower aarch64 serror");
}

#[no_mangle]
extern "C" fn lower_aarch32_synchronous(e: &mut ExceptionContext) {
    panic!("lower aarch32 synchronous");
}

#[no_mangle]
extern "C" fn lower_aarch32_irq(e: &mut ExceptionContext) {
    panic!("lower aarch32 irq");
}

#[no_mangle]
extern "C" fn lower_aarch32_serror(e: &mut ExceptionContext) {
    panic!("lower aarch32 serror");
}

pub unsafe fn init_cpu_exceptions() {
    extern "Rust" {
        static __exception_vector_start: UnsafeCell<()>;
    }
    VBAR_EL2.set(__exception_vector_start.get() as u64);
    barrier::isb(barrier::SY);
}
