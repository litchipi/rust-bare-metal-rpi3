use core::arch::{asm, global_asm};
use core::time::Duration;

use aarch64_cpu::registers::DAIF;
use alloc::{boxed::Box, collections::BTreeMap};
use tock_registers::register_bitfields;
use tock_registers::registers::ReadWrite;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_structs,
    registers::{ReadOnly, WriteOnly},
};

use crate::drivers::timer::Counter;
use crate::memory::{MMIODerefWrapper, IRQ_BASE};
use crate::{println, dbg};
use crate::sync::RwLock;

pub static IRQ_MANAGER: IrqManager = IrqManager::init();

#[derive(Copy, Clone)]
pub struct IrqHandlerDescriptor {
    pub handler: &'static (dyn IrqHandler + Sync),
}

#[derive(Debug)]
pub enum IrqNumber {
    Basic(usize),
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
    basic_irq_table: RwLock<IrqHandlerTable>,
    periph_irq_table: RwLock<IrqHandlerTable>,
    regs: RwLock<MMIODerefWrapper<IrqRegister>>,
}

impl IrqManager {
    const fn init() -> IrqManager {
        IrqManager {
            basic_irq_table: RwLock::new(BTreeMap::new()),
            periph_irq_table: RwLock::new(BTreeMap::new()),
            regs: RwLock::new(MMIODerefWrapper::new(IRQ_BASE)),
        }
    }

    pub fn register<H: IrqHandler + Sync>(&self, irq_number: IrqNumber, handler: &'static H) {
        dbg!("Register handle for IRQ {irq_number:?}\n");
        match irq_number {
            IrqNumber::Basic(n) => {
                self.basic_irq_table.write(|table| table.insert(n, handler));
            }
            IrqNumber::Peripheral(n) => {
                self.periph_irq_table
                    .write(|table| table.insert(n, handler));
            }
        }
    }

    pub fn enable(&self, irq_number: IrqNumber) {
        dbg!("Enabling IRQ {irq_number:?}\n");
        self.regs.write(|regs| match irq_number {
            IrqNumber::Basic(nb) => {
                dbg!("DBG Set value {} to BASIC_ENABLE", 1 << nb);
                // TODO    FIXME        Fix IRQ registering
                unsafe {
                    core::ptr::write_volatile((0x4000_0040) as *mut _, 1 << nb);
                }
                regs.BASIC_ENABLE.set(1 << nb);
            },
            IrqNumber::Peripheral(nb) => if nb <= 31 {
                dbg!("DBG Set value {} to ENABLE_1", 1 << nb);
                regs.ENABLE_1.set(1 << nb);
            } else {
                dbg!("DBG Set value {} to ENABLE_2", 1 << (nb % 32));
                regs.ENABLE_2.set(1 << (nb % 32));
            },
        });
    }

    pub(crate) fn handle_pending_irqs(&self) {
        self.handle_basic_irqs();
        self.handle_periph_irqs();
    }

    fn handle_basic_irqs(&self) {
        // Ignore the indicator bit for a peripheral IRQ.
        let periph_irq_mask = !(1 << 8);
        let pending_mask = (self.regs.read(|regs| regs.BASIC_PENDING.get()) & periph_irq_mask).into();
        for irq_number in PendingIRQs::new(pending_mask) {
            println!("Basic irq {irq_number}");
            match self.basic_irq_table.read(|t| t.get(&irq_number)) {
                None => panic!("No handler registered for IRQ {}", irq_number),
                Some(handler) => {
                    handler.handle().expect("Error handling IRQ");
                }
            }
        }
    }

    fn handle_periph_irqs(&self) {
        let pending_mask: u64 = self.regs.read(|regs|
            (u64::from(regs.PENDING_2.get()) << 32) | u64::from(regs.PENDING_1.get())
        );

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

register_bitfields! {
    u32,

    FIQ_CONTROL [
        ENABLE OFFSET(7) NUMBITS(1) [],
        SOURCE OFFSET(0) NUMBITS(7) [
            ArmTimer = 64,
            ArmMailbox = 65,
            ArmDoorbell1 = 66,
            ArmDoorbell2 = 67,
            Gpu0Halted = 68,
            Gpu1Halted = 69,
            IllegalAccessType1 = 70,
            IllegalAccessType0 = 71,
        ],
    ],
    ENABLE_1 [
        IRQ OFFSET(0) NUMBITS(32) [
            SystemTimerMatch1 = 1 << 1,
            SystemTimerMatch3 = 1 << 3,
            UsbController = 1 << 9,
            AuxInt = 1 << 29,
        ]
    ],
    ENABLE_2 [
        IRQ OFFSET(0) NUMBITS(32) [
            I2cSpiSlvInt = 1 << (43 - 32),
            Pwa0 = 1 << (45 - 32),
            Pwa1 = 1 << (46 - 32),
            Smi = 1 << (48 - 32),
            GpioInt0 = 1 << (49 - 32),
            GpioInt1 = 1 << (50 - 32),
            GpioInt2 = 1 << (51 - 32),
            GpioInt3 = 1 << (52 - 32),
            I2cInt = 1 << (53 - 32),
            SpiInt = 1 << (54 - 32),
            PcmInt = 1 << (55 - 32),
            UartInt = 1 << (57 - 32),
        ]
    ],
    BASIC_ENABLE [
        IRQ OFFSET(0) NUMBITS(8) [
            ArmTimer = 1,
            ArmMailbox = 1 << 1,
            ArmDoorbell0 = 1 << 2,
            ArmDoorbell1 = 1 << 3,
            Gpu0Halted = 1 << 4,
            Gpu1Halted = 1 << 5,
            AccessErrorType1 = 1 << 6,
            AccessErrorType0 = 1 << 7,
        ]
    ],
    DISABLE_1 [
        IRQ OFFSET(0) NUMBITS(32) [
            SystemTimerMatch1 = 1 << 1,
            SystemTimerMatch3 = 1 << 3,
            UsbController = 1 << 9,
            AuxInt = 1 << 29,
        ]
    ],
    DISABLE_2 [
        IRQ OFFSET(0) NUMBITS(32) [
            I2cSpiSlvInt = 1 << (43 - 32),
            Pwa0 = 1 << (45 - 32),
            Pwa1 = 1 << (46 - 32),
            Smi = 1 << (48 - 32),
            GpioInt0 = 1 << (49 - 32),
            GpioInt1 = 1 << (50 - 32),
            GpioInt2 = 1 << (51 - 32),
            GpioInt3 = 1 << (52 - 32),
            I2cInt = 1 << (53 - 32),
            SpiInt = 1 << (54 - 32),
            PcmInt = 1 << (55 - 32),
            UartInt = 1 << (57 - 32),
        ]
    ],
    BASIC_DISABLE [
        IRQ OFFSET(0) NUMBITS(8) [
            ArmTimer = 1,
            ArmMailbox = 1 << 1,
            ArmDoorbell0 = 1 << 2,
            ArmDoorbell1 = 1 << 3,
            Gpu0Halted = 1 << 4,
            Gpu1Halted = 1 << 5,
            AccessErrorType1 = 1 << 6,
            AccessErrorType0 = 1 << 7,
        ]
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    IrqRegister {
        (0x00 => BASIC_PENDING: ReadOnly<u32>),
        (0x04 => PENDING_1: ReadOnly<u32>),
        (0x08 => PENDING_2: ReadOnly<u32>),
        (0x0C => FIQ_CONTROL: ReadWrite<u32, FIQ_CONTROL::Register>),

        (0x10 => ENABLE_1: WriteOnly<u32, ENABLE_1::Register>),
        (0x14 => ENABLE_2: WriteOnly<u32, ENABLE_2::Register>),
        (0x18 => BASIC_ENABLE: WriteOnly<u32, BASIC_ENABLE::Register>),

        (0x1C => DISABLE_1: WriteOnly<u32, DISABLE_1::Register>),
        (0x20 => DISABLE_2: WriteOnly<u32, DISABLE_2::Register>),
        (0x24 => BASIC_DISABLE: WriteOnly<u32, BASIC_DISABLE::Register>),
        (0x28 => @END),
    }
}
