use aarch64_cpu::registers::{ESR_EL2, SPSR_EL2};
use tock_registers::{interfaces::Readable, registers::InMemoryRegister};

struct SpsrEL2(InMemoryRegister<u64, SPSR_EL2::Register>);
struct EsrEL2(InMemoryRegister<u64, ESR_EL2::Register>);

impl EsrEL2 {
    #[inline(always)]
    fn exception_class(&self) -> Option<ESR_EL2::EC::Value> {
        self.0.read_as_enum(ESR_EL2::EC)
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
    elr_el2: u64,

    /// Saved program status.
    spsr_el2: SpsrEL2,

    // Exception syndrome register.
    esr_el2: EsrEL2,
}

#[no_mangle]
extern "C" fn current_el0_synchronous(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL2 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_irq(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL2 is not supported.")
}

#[no_mangle]
extern "C" fn current_el0_serror(_e: &mut ExceptionContext) {
    panic!("Should not be here. Use of SP_EL0 in EL2 is not supported.")
}

#[no_mangle]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    crate::irq::IRQ_MANAGER.handle_pending_irqs();
    panic!("current elx irq");
}

#[no_mangle]
extern "C" fn current_elx_synchronous(e: &mut ExceptionContext) {
    // TODO    Create handler for memory faults here using the exception context
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
