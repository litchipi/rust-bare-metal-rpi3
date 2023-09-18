#[cfg(not(feature = "builder"))]
use core::arch::global_asm;
use core::time::Duration;

use tock_registers::{
    interfaces::{ReadWriteable, Writeable},
    register_structs,
    registers::{ReadOnly, WriteOnly},
};

use crate::{sync::RwLock, timer::Counter};

pub static IRQ_MANAGER: IrqManager = IrqManager::init();

type HandlerTable<const N: usize> = [Option<IrqHandlerDescriptor>; N];

const LOCAL_IRQ_COUNT: usize = 15;
const PERIPH_IRQ_COUNT: usize = 20;

#[derive(Copy, Clone)]
pub struct IrqHandlerDescriptor {
    number: usize,
    handler: &'static (dyn IrqHandler + Sync),
}

pub enum IrqNumber {
    Local(usize),
    Peripheral(usize),
}

pub struct IrqManager {
    local_irq_table: RwLock<HandlerTable<LOCAL_IRQ_COUNT>>,
    periph_irq_table: RwLock<HandlerTable<PERIPH_IRQ_COUNT>>,
}

impl IrqManager {
    const fn init() -> IrqManager {
        IrqManager {
            local_irq_table: RwLock::new([None; LOCAL_IRQ_COUNT]),
            periph_irq_table: RwLock::new([None; PERIPH_IRQ_COUNT]),
        }
    }

    pub fn register<H: IrqHandler + Sync>(&self, irq_number: IrqNumber, handler: &'static H) {
        match irq_number {
            IrqNumber::Local(n) => {
                assert!(n < LOCAL_IRQ_COUNT);
                self.local_irq_table
                    .write(|table| table[n] = Some(IrqHandlerDescriptor { number: n, handler }));
            }
            IrqNumber::Peripheral(n) => {
                assert!(n < PERIPH_IRQ_COUNT);
                self.periph_irq_table
                    .write(|table| table[n] = Some(IrqHandlerDescriptor { number: n, handler }));
            }
        }
    }

    pub(crate) fn handle_pending_irqs(&self) {
        // TODO    Implement this function
        todo!();
    }
}

pub trait IrqHandler {
    fn handle(&self) -> Result<(), &'static str>;
}

register_structs! {
    #[allow(non_snake_case)]
    WORegisterBlock {
        (0x00 => _reserved1),
        (0x10 => ENABLE_1: WriteOnly<u32>),
        (0x14 => ENABLE_2: WriteOnly<u32>),
        (0x18 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    RORegisterBlock {
        (0x00 => _reserved1),
        (0x04 => PENDING_1: ReadOnly<u32>),
        (0x08 => PENDING_2: ReadOnly<u32>),
        (0x0c => @END),
    }
}

pub fn local_irq_unmask() {
    todo!();
}

pub fn exec_with_irq_masked<T>(f: impl FnOnce() -> T) -> T {
    // TODO    Implement exec with irq masked
    // let saved = local_irq_mask_save();
    let ret = f();
    // local_irq_restore(saved);

    ret
}

#[cfg(not(feature = "builder"))]
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
