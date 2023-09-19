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
use crate::println;
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
        match irq_number {
            IrqNumber::Local(nb) => self.lreg_w.write(|regs| {
                let enable_bit: u32 = 1 << nb;
                regs.CORE0_TIMER_INTERRUPT_CONTROL.set(enable_bit);
            }),
            IrqNumber::Peripheral(nb) => self.preg_w.write(|regs| {
                let enable_reg = if nb <= 31 {
                    &regs.ENABLE_1
                } else {
                    &regs.ENABLE_2
                };

                // Writing a 1 to a bit will set the corresponding IRQ enable bit.
                // All other IRQ enable bits are unaffected. So we don't need read and OR'ing here.
                enable_reg.set(1 << (nb % 32));
            }),
        }
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
            println!("Local irq {irq_number}");
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
            println!("Periph irq {irq_number}");
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
