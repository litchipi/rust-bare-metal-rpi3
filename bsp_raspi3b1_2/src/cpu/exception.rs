use core::arch::global_asm;

use aarch64_cpu::registers::{ESR_EL2, SPSR_EL2};
use tock_registers::{interfaces::Readable, registers::InMemoryRegister};

use crate::{println, dbg};

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
    // TODO    FIXME        Try to force IRQ generation and check if this get executed
    dbg!("Got IRQ exception");
    crate::irq::IRQ_MANAGER.handle_pending_irqs();
    panic!("IRQ handled");
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

global_asm!(
    r"

/// Call the function provided by parameter `\handler` after saving the exception context.
/// Provide the context as the first parameter to '\handler'.
.macro CALL_WITH_CONTEXT handler
    __vector_\handler:
        // Make room on the stack for the exception context.
        sub    sp,  sp,  #16 * 17

        // Store all general purpose registers on the stack.
        stp    x0,  x1,  [sp, #16 * 0]
        stp    x2,  x3,  [sp, #16 * 1]
        stp    x4,  x5,  [sp, #16 * 2]
        stp    x6,  x7,  [sp, #16 * 3]
        stp    x8,  x9,  [sp, #16 * 4]
        stp    x10, x11, [sp, #16 * 5]
        stp    x12, x13, [sp, #16 * 6]
        stp    x14, x15, [sp, #16 * 7]
        stp    x16, x17, [sp, #16 * 8]
        stp    x18, x19, [sp, #16 * 9]
        stp    x20, x21, [sp, #16 * 10]
        stp    x22, x23, [sp, #16 * 11]
        stp    x24, x25, [sp, #16 * 12]
        stp    x26, x27, [sp, #16 * 13]
        stp    x28, x29, [sp, #16 * 14]

        // Add the exception link register (ELR_EL2), saved program status (SPSR_EL2) and exception
        // syndrome register (ESR_EL2).
        mrs    x1,  ELR_EL2
        mrs    x2,  SPSR_EL2
        mrs    x3,  ESR_EL2

        stp    lr,  x1,  [sp, #16 * 15]
        stp    x2,  x3,  [sp, #16 * 16]

        // x0 is the first argument for the function called through `\handler`.
        mov    x0,  sp

        // Call `\handler`.
        bl    \handler

        // After returning from exception handling code, replay the saved context and return via
        // `eret`.
        b    __exception_restore_context

    .size    __vector_\handler, . - __vector_\handler
    .type    __vector_\handler, function
.endm

.macro FIQ_SUSPEND
    1:    wfe
          b 1b
.endm

.section .text

.align 11

__exception_vector_start:
    .org 0x000
    	CALL_WITH_CONTEXT current_el0_synchronous
    .org 0x080
    	CALL_WITH_CONTEXT current_el0_irq
    .org 0x100
    	FIQ_SUSPEND
    .org 0x180
    	CALL_WITH_CONTEXT current_el0_serror

    // Current exception level with SP_ELx, x > 0.
    .org 0x200
    	CALL_WITH_CONTEXT current_elx_synchronous
    .org 0x280
    	CALL_WITH_CONTEXT current_elx_irq
    .org 0x300
    	FIQ_SUSPEND
    .org 0x380
    	CALL_WITH_CONTEXT current_elx_serror

    // Lower exception level, AArch64
    .org 0x400
    	CALL_WITH_CONTEXT lower_aarch64_synchronous
    .org 0x480
    	CALL_WITH_CONTEXT lower_aarch64_irq
    .org 0x500
    	FIQ_SUSPEND
    .org 0x580
    	CALL_WITH_CONTEXT lower_aarch64_serror

    // Lower exception level, AArch32
    .org 0x600
    	CALL_WITH_CONTEXT lower_aarch32_synchronous
    .org 0x680
    	CALL_WITH_CONTEXT lower_aarch32_irq
    .org 0x700
    	FIQ_SUSPEND
    .org 0x780
    	CALL_WITH_CONTEXT lower_aarch32_serror
    .org 0x800

__exception_restore_context:
    ldr    w19,      [sp, #16 * 16]
    ldp    lr,  x20, [sp, #16 * 15]

    msr    SPSR_EL2, x19
    msr    ELR_EL2,  x20

    ldp    x0,  x1,  [sp, #16 * 0]
    ldp    x2,  x3,  [sp, #16 * 1]
    ldp    x4,  x5,  [sp, #16 * 2]
    ldp    x6,  x7,  [sp, #16 * 3]
    ldp    x8,  x9,  [sp, #16 * 4]
    ldp    x10, x11, [sp, #16 * 5]
    ldp    x12, x13, [sp, #16 * 6]
    ldp    x14, x15, [sp, #16 * 7]
    ldp    x16, x17, [sp, #16 * 8]
    ldp    x18, x19, [sp, #16 * 9]
    ldp    x20, x21, [sp, #16 * 10]
    ldp    x22, x23, [sp, #16 * 11]
    ldp    x24, x25, [sp, #16 * 12]
    ldp    x26, x27, [sp, #16 * 13]
    ldp    x28, x29, [sp, #16 * 14]

    add    sp,  sp,  #16 * 17

    eret

.size    __exception_restore_context, . - __exception_restore_context
.type    __exception_restore_context, function

"
);
