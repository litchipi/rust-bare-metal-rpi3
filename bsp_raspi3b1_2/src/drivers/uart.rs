use aarch64_cpu::asm;
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use crate::{
    memory::{MMIODerefWrapper, UART0_BASE},
    sync::NullLock,
};

use super::gpio::PinMode;

pub static UART: UartDriver = UartDriver::init();

pub struct UartDriver {
    registers: NullLock<Registers>,
    pub init: NullLock<bool>,
}

impl UartDriver {
    const fn init() -> UartDriver {
        UartDriver {
            registers: NullLock::new(Registers::new(UART0_BASE)),
            init: NullLock::new(false),
        }
    }

    pub fn flush(&self) {
        // Spin until the busy bit is cleared.
        loop {
            let busy = self.registers.lock(|reg| reg.FR.matches_all(FR::BUSY::SET));
            if busy {
                asm::nop();
            } else {
                break;
            }
        }
    }

    pub fn configure(&self, txd: usize, rxd: usize) {
        let gpios = &super::GPIO;
        gpios.configure(&[(txd, PinMode::UartTxd(0)), (rxd, PinMode::UartRxd(0))]);
        gpios.disable_pud(&[txd, rxd]);
        self.flush();
        self.registers.lock(|reg| {
            reg.CR.set(0); // Turn the UART off temporarily.
            reg.ICR.write(ICR::ALL::CLEAR); // Clear all pending interrupts.

            // Set the baud rate, 8N1 and FIFO enabled.
            reg.IBRD.write(IBRD::BAUD_DIVINT.val(3));
            reg.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
            reg.LCR_H
                .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);

            // Turn the UART on.
            reg.CR
                .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
        });
        self.init.lock(|i| *i = true);
    }

    pub fn write(&self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }

    pub fn write_char(&self, c: char) {
        while self.registers.lock(|reg| reg.FR.matches_all(FR::TXFF::SET)) {
            asm::nop();
        }
        self.registers.lock(|reg| reg.DR.set(c as u32));
    }

    pub fn read_byte(&self, blocking: bool) -> Option<u8> {
        while self.registers.lock(|reg| reg.FR.matches_all(FR::RXFE::SET)) {
            if !blocking {
                return None;
            }
            asm::nop();
        }
        let ret = self.registers.lock(|reg| reg.DR.get()) as u8;
        Some(ret)
    }

    pub fn read_char(&self, blocking: bool) -> Option<char> {
        self.read_byte(blocking).map(|res| res as char)
    }

    pub fn clear_rx(&self) {
        while let Some(c) = self.read_char(false) {}
    }
}

register_bitfields! {
    u32,

    /// Flag Register.
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// Line Control Register, LCR_H.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is empty.
        /// - If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty.
        /// - This bit does not indicate if there is data in the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1) [],

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the transmit holding register is full.
        /// - If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1) [],

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// LCR_H Register.
        ///
        /// - If the FIFO is disabled, this bit is set when the receive holding register is empty.
        /// - If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1) [],

        /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains
        /// set until the complete byte, including all the stop bits, has been sent from the shift
        /// register.
        ///
        /// This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether
        /// the UART is enabled or not.
        BUSY OFFSET(3) NUMBITS(1) []
    ],

    /// Integer Baud Rate Divisor.
    IBRD [
        /// The integer baud rate divisor.
        BAUD_DIVINT OFFSET(0) NUMBITS(16) []
    ],

    /// Fractional Baud Rate Divisor.
    FBRD [
        ///  The fractional baud rate divisor.
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) []
    ],

    /// Line Control Register.
    LCR_H [
        /// Word length. These bits indicate the number of data bits transmitted or received in a
        /// frame.
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11
        ],

        /// Enable FIFOs:
        ///
        /// 0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep holding
        /// registers.
        ///
        /// 1 = Transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN  OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1
        ]
    ],

    /// Control Register.
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        /// Data reception occurs for either UART signals or SIR signals depending on the setting of
        /// the SIREN bit. When the UART is disabled in the middle of reception, it completes the
        /// current character before stopping.
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled.
        /// Data transmission occurs for either UART signals, or SIR signals depending on the
        /// setting of the SIREN bit. When the UART is disabled in the middle of transmission, it
        /// completes the current character before stopping.
        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1
        ],

        /// UART enable:
        ///
        /// 0 = UART is disabled. If the UART is disabled in the middle of transmission or
        /// reception, it completes the current character before stopping.
        ///
        /// 1 = The UART is enabled. Data transmission and reception occurs for either UART signals
        /// or SIR signals depending on the setting of the SIREN bit
        UARTEN OFFSET(0) NUMBITS(1) [
            /// If the UART is disabled in the middle of transmission or reception, it completes the
            /// current character before stopping.
            Disabled = 0,
            Enabled = 1
        ]
    ],

    /// Interrupt Clear Register.
    ICR [
        /// Meta field for all pending interrupts.
        ALL OFFSET(0) NUMBITS(11) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type Registers = MMIODerefWrapper<RegisterBlock>;
