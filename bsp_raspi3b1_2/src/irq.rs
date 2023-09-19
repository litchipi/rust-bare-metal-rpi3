use core::arch::{asm, global_asm};
use core::time::Duration;

use aarch64_cpu::registers::DAIF;
use alloc::{boxed::Box, collections::BTreeMap};
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_structs,
    registers::{ReadOnly, WriteOnly},
};

use crate::drivers::timer::Counter;
use crate::memory::{MMIODerefWrapper, LOCAL_IRQ_START, PERIPH_IRQ_START};
use crate::sync::RwLock;

pub static IRQ_MANAGER: IrqManager = IrqManager::init();

#[derive(Copy, Clone)]
pub struct IrqHandlerDescriptor {
    pub handler: &'static (dyn IrqHandler + Sync),
}

pub enum IrqNumber {
    Local(usize),
    Peripheral(usize),
}

struct PendingIRQs {
    bitmask: u64,
}

impl PendingIRQs {
    pub fn new(bitmask: u64) -> Self {
        Self { bitmask }
    }
}

impl Iterator for PendingIRQs {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitmask == 0 {
            return None;
        }

        let next = self.bitmask.trailing_zeros() as usize;
        self.bitmask &= self.bitmask.wrapping_sub(1);
        Some(next)
    }
}

type IrqHandlerTable = BTreeMap<usize, &'static dyn IrqHandler>;

pub struct IrqManager {
    local_irq_table: RwLock<IrqHandlerTable>,
    periph_irq_table: RwLock<IrqHandlerTable>,

    lreg_r: MMIODerefWrapper<RLocalIrqRegister>,
    lreg_w: RwLock<MMIODerefWrapper<WLocalIrqRegister>>,

    preg_r: MMIODerefWrapper<RPeriphIrqRegister>,
    preg_w: RwLock<MMIODerefWrapper<WPeriphIrqRegister>>,
}

impl IrqManager {
    const fn init() -> IrqManager {
        IrqManager {
            local_irq_table: RwLock::new(BTreeMap::new()),
            periph_irq_table: RwLock::new(BTreeMap::new()),
            lreg_r: MMIODerefWrapper::new(LOCAL_IRQ_START),
            lreg_w: RwLock::new(MMIODerefWrapper::new(LOCAL_IRQ_START)),
            preg_r: MMIODerefWrapper::new(PERIPH_IRQ_START),
            preg_w: RwLock::new(MMIODerefWrapper::new(PERIPH_IRQ_START)),
        }
    }

    pub fn register<H: IrqHandler + Sync>(&self, irq_number: IrqNumber, handler: &'static H) {
        match irq_number {
            IrqNumber::Local(n) => {
                self.local_irq_table.write(|table| table.insert(n, handler));
            }
            IrqNumber::Peripheral(n) => {
                self.periph_irq_table
                    .write(|table| table.insert(n, handler));
            }
        }
    }

    pub fn enable(&self, irq_number: IrqNumber) {
        // TODO    Enable IRQ
        todo!();
    }

    pub(crate) fn handle_pending_irqs(&self) {
        self.handle_local_irqs();
        self.handle_periph_irqs();
    }

    fn handle_local_irqs(&self) {
        // Ignore the indicator bit for a peripheral IRQ.
        let periph_irq_mask = !(1 << 8);
        let pending_mask = (self.lreg_r.CORE0_INTERRUPT_SOURCE.get() & periph_irq_mask).into();
        for irq_number in PendingIRQs::new(pending_mask) {
            match self.local_irq_table.read(|t| t.get(&irq_number)) {
                None => panic!("No handler registered for IRQ {}", irq_number),
                Some(handler) => {
                    handler.handle().expect("Error handling IRQ");
                }
            }
        }
    }

    fn handle_periph_irqs(&self) {
        let pending_mask: u64 =
            (u64::from(self.preg_r.PENDING_2.get()) << 32) | u64::from(self.preg_r.PENDING_1.get());

        for irq_number in PendingIRQs::new(pending_mask) {
            match self.periph_irq_table.read(|t| t.get(&irq_number)) {
                None => panic!("No handler registered for IRQ {}", irq_number),
                Some(handler) => {
                    handler.handle().expect("Error handling IRQ");
                }
            }
        }
    }
}

pub trait IrqHandler: Sync {
    fn handle(&self) -> Result<(), &'static str>;
}

pub fn exec_with_irq_masked<T>(f: impl FnOnce() -> T) -> T {
    let saved = local_irq_mask_save();
    let ret = f();
    local_irq_restore(saved);
    ret
}

#[inline(always)]
pub fn local_irq_unmask() {
    unsafe {
        asm!("msr DAIFClr, 2", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn local_irq_mask() {
    unsafe {
        asm!("msr DAIFSet, 2", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn local_irq_mask_save() -> u64 {
    let saved = DAIF.get();
    local_irq_mask();

    saved
}

#[inline(always)]
pub fn local_irq_restore(saved: u64) {
    DAIF.set(saved);
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

register_structs! {
    #[allow(non_snake_case)]
    WPeriphIrqRegister {
        (0x00 => _reserved1),
        (0x10 => ENABLE_1: WriteOnly<u32>),
        (0x14 => ENABLE_2: WriteOnly<u32>),
        (0x18 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    RPeriphIrqRegister {
        (0x00 => _reserved1),
        (0x04 => PENDING_1: ReadOnly<u32>),
        (0x08 => PENDING_2: ReadOnly<u32>),
        (0x0c => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    WLocalIrqRegister {
        (0x00 => _reserved1),
        (0x40 => CORE0_TIMER_INTERRUPT_CONTROL: WriteOnly<u32>),
        (0x44 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    RLocalIrqRegister {
        (0x00 => _reserved1),
        (0x60 => CORE0_INTERRUPT_SOURCE: ReadOnly<u32>),
        (0x64 => @END),
    }
}
