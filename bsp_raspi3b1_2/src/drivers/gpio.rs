use core::time::Duration;

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};
use tock_registers::{register_bitfields, register_structs};

use crate::memory::{MMIODerefWrapper, GPIO_BASE};
use crate::sync::NullLock;
use crate::timer::spin_for;

const TOT_NUMBER_GPIO: usize = 54;
pub static GPIO: GpioDriver = GpioDriver::init();

pub struct GpioDriver {
    registers: NullLock<GpioRegisters>,
}

impl GpioDriver {
    const fn init() -> GpioDriver {
        GpioDriver {
            registers: NullLock::new(GpioRegisters::new(GPIO_BASE)),
        }
    }

    pub fn panic_led_on(&self) {
        self.registers.lock(|reg| {
            reg.GPFSEL2.write(GPFSEL2::FSEL21::Output);
            reg.GPSET0.set(1 << 21);
        })
    }

    pub fn configure(&self, config: &[(usize, PinMode)]) {
        let mut used_pins = [false; TOT_NUMBER_GPIO];
        let mut gpfsel: [u32; 6] = [0, 0, 0, 0, 0, 0];
        self.registers.lock(|reg| {
            gpfsel[0] = reg.GPFSEL0.get();
            gpfsel[1] = reg.GPFSEL1.get();
            gpfsel[2] = reg.GPFSEL2.get();
            gpfsel[3] = reg.GPFSEL3.get();
            gpfsel[4] = reg.GPFSEL4.get();
            gpfsel[5] = reg.GPFSEL5.get();
        });

        for (pin_nb, mode) in config {
            let fsel_idx = pin_nb / 10;
            let fsel_offset = (pin_nb % 10) * 3;
            let val = mode.get_value(*pin_nb);
            let used_pins_idx: usize = *pin_nb;
            assert!(
                !used_pins[used_pins_idx],
                "Pin already configured for something else"
            );
            used_pins[used_pins_idx] = true;
            gpfsel[fsel_idx] |= val << fsel_offset;
        }

        self.registers.lock(|reg| {
            reg.GPFSEL0.set(gpfsel[0]);
            reg.GPFSEL1.set(gpfsel[1]);
            reg.GPFSEL2.set(gpfsel[2]);
            reg.GPFSEL3.set(gpfsel[3]);
            reg.GPFSEL4.set(gpfsel[4]);
            reg.GPFSEL5.set(gpfsel[5]);
        })
    }

    pub fn set_pin(&self, nb: usize) {
        assert!(nb < TOT_NUMBER_GPIO);
        self.registers.lock(|reg| {
            if nb < 32 {
                reg.GPSET0.set(1 << nb);
                reg.GPSET0.set(0);
            } else {
                reg.GPSET1.set(1 << (nb - 32));
                reg.GPSET1.set(0);
            }
        })
    }

    pub fn clear_pin(&self, nb: usize) {
        assert!(nb < TOT_NUMBER_GPIO);
        self.registers.lock(|reg| {
            if nb < 32 {
                reg.GPCLR0.set(1 << nb);
                reg.GPCLR0.set(0);
            } else {
                reg.GPCLR1.set(1 << (nb - 32));
                reg.GPCLR1.set(0);
            }
        })
    }

    pub fn get_pin_state(&self, nb: usize) -> bool {
        assert!(nb < TOT_NUMBER_GPIO);
        self.registers.lock(|reg| {
            if nb < 32 {
                (reg.GPLEV0.get() | (1 << nb)) > 0
            } else {
                (reg.GPLEV1.get() | (1 << (nb - 32))) > 0
            }
        })
    }

    pub fn disable_pud(&self, pins: &[usize]) {
        assert!(pins.iter().all(|nb| *nb < TOT_NUMBER_GPIO));
        self.registers.lock(|reg| {
            reg.GPPUD.set(0);
            spin_for(Duration::from_micros(10));
            let mut val0 = 0u32;
            let mut val1 = 0u32;
            for nb in pins.iter() {
                if *nb < 32 {
                    val0 |= 1 << nb;
                } else {
                    val1 |= 1 << (nb - 32);
                }
            }
            reg.GPPUDCLK0.set(val0);
            reg.GPPUDCLK1.set(val1);

            spin_for(Duration::from_micros(10));
            reg.GPPUD.set(0);
            reg.GPPUDCLK0.set(0);
            reg.GPPUDCLK1.set(0);
        })
    }
}

#[derive(Debug)]
pub enum PinMode {
    Input,
    Output,
    BscSda(usize),
    BscScl(usize),
    GpClk(usize),
    SpiCs(usize, usize),
    SpiMiso(usize),
    SpiMosi(usize),
    SpiSclk(usize),
    Pwm(usize),
    UartTxd(usize),
    UartRxd(usize),
    UartCts(usize),
    UartRts(usize),
    PcmClk,
    PcmFs,
    PcmDin,
    PcmDout,
    SmiSa(usize),
    SmiSoeNSe,
    SmiSweNSrwN,
    SmiSd(usize),
    BscSlSda,
    BscSlScl,
    SpiSlMosi,
    SpiSlMiso,
    SpiSlSclk,
    SpiSlCs,
    JtagTrst,
    JtagRtck,
    JtagTdo,
    JtagTck,
    JtagTdi,
    JtagTms,
}

impl PinMode {
    pub fn get_value(&self, pin_nb: usize) -> u32 {
        match self {
            PinMode::Input => 0b000,
            PinMode::Output => 0b001,
            PinMode::BscSda(n) => match (n, pin_nb) {
                (0, 0) | (1, 2) | (0, 28) => 0b100,
                (0, 44) => 0b101,
                (1, 44) => 0b110,
                arg => unreachable!("BscSda {arg:?}"),
            },
            PinMode::BscScl(n) => match (n, pin_nb) {
                (0, 1) | (1, 3) | (0, 29) => 0b100,
                (0, 45) => 0b101,
                (1, 45) => 0b110,
                arg => unreachable!("BscScl {arg:?}"),
            },
            PinMode::GpClk(n) => match (n, pin_nb) {
                (0, 4) | (1, 5) | (2, 6) | (0, 32) | (0, 34) | (1, 42) | (2, 43) | (1, 44) => 0b100,
                (0, 20) | (1, 21) => 0b010,
                arg => unreachable!("GpClk {arg:?}"),
            },
            PinMode::SpiCs(spin, csn) => match (spin, csn, pin_nb) {
                (0, 1, 7) | (0, 0, 8) | (0, 1, 35) | (0, 0, 36) => 0b100,
                (1, 0, 18) | (1, 2, 16) | (1, 1, 17) => 0b011,
                arg => unreachable!("SpiCs {arg:?}"),
            },
            PinMode::SpiMiso(n) => match (n, pin_nb) {
                (0, 9) | (0, 37) => 0b100,
                (1, 19) => 0b011,
                arg => unreachable!("SpiMiso {arg:?}"),
            },
            PinMode::SpiMosi(n) => match (n, pin_nb) {
                (0, 10) | (0, 38) => 0b100,
                (1, 20) => 0b011,
                arg => unreachable!("SpiMosi {arg:?}"),
            },
            PinMode::SpiSclk(n) => match (n, pin_nb) {
                (0, 39) | (0, 11) => 0b100,
                (1, 21) => 0b011,
                arg => unreachable!("SpiSclk {arg:?}"),
            },
            PinMode::Pwm(n) => match (n, pin_nb) {
                (0, 12) | (1, 13) | (0, 40) | (1, 41) | (1, 45) => 0b100,
                (0, 18) | (1, 19) => 0b010,
                arg => unreachable!("Pwm {arg:?}"),
            },
            PinMode::UartTxd(n) => match (n, pin_nb) {
                (0, 14) => 0b100,
                (0, 32) => 0b111,
                (0, 36) => 0b110,
                (1, 14) | (1, 32) | (1, 40) => 0b010,
                arg => unreachable!("UartTxd {arg:?}"),
            },
            PinMode::UartRxd(n) => match (n, pin_nb) {
                (0, 15) => 0b100,
                (1, 15) | (1, 33) | (1, 40) => 0b010,
                (0, 37) => 0b110,
                (0, 33) => 0b111,
                arg => unreachable!("UartRxd {arg:?}"),
            },
            PinMode::UartCts(n) => match (n, pin_nb) {
                (0, 16) | (0, 30) => 0b111,
                (1, 16) | (1, 30) | (1, 43) => 0b010,
                (0, 39) => 0b110,
                arg => unreachable!("UartCts {arg:?}"),
            },
            PinMode::UartRts(n) => match (n, pin_nb) {
                (0, 17) | (0, 31) => 0b111,
                (1, 17) | (1, 31) | (1, 42) => 0b010,
                (0, 38) => 0b110,
                arg => unreachable!("UartRts {arg:?}"),
            },
            PinMode::PcmClk => match pin_nb {
                18 => 0b100,
                28 => 0b110,
                arg => unreachable!("PcmClk {arg:?}"),
            },
            PinMode::PcmFs => match pin_nb {
                19 => 0b100,
                29 => 0b110,
                arg => unreachable!("PcmFs {arg:?}"),
            },
            PinMode::PcmDin => match pin_nb {
                20 => 0b100,
                30 => 0b110,
                arg => unreachable!("PcmDin {arg:?}"),
            },
            PinMode::PcmDout => match pin_nb {
                21 => 0b100,
                31 => 0b110,
                arg => unreachable!("PcmDout {arg:?}"),
            },
            PinMode::SmiSa(n) => match (n, pin_nb) {
                (5, 0) | (4, 1) | (3, 2) | (2, 3) | (1, 4) | (0, 5) => 0b101,
                (5, 28) | (4, 29) | (3, 30) | (2, 31) | (1, 32) | (0, 33) => 0b101,
                arg => unreachable!("SmiSa {arg:?}"),
            },
            PinMode::SmiSoeNSe => match pin_nb {
                6 => 0b101,
                34 => 0b101,
                arg => unreachable!("SmiSoeNSe {arg:?}"),
            },
            PinMode::SmiSweNSrwN => match pin_nb {
                7 => 0b101,
                35 => 0b101,
                arg => unreachable!("SmiSweNSrwN {arg:?}"),
            },
            PinMode::SmiSd(n) => {
                if (pin_nb == (8 + *n)) || (pin_nb == (36 + *n)) {
                    0b101
                } else {
                    unreachable!("SmiSd ({n}, {pin_nb})")
                }
            }
            PinMode::BscSlSda | PinMode::SpiSlMosi => {
                if pin_nb == 18 {
                    0b111
                } else {
                    unreachable!("{self:?} {pin_nb}")
                }
            }
            PinMode::BscSlScl | PinMode::SpiSlSclk => {
                if pin_nb == 19 {
                    0b111
                } else {
                    unreachable!("{self:?} {pin_nb}")
                }
            }
            PinMode::SpiSlMiso => {
                if pin_nb == 20 {
                    0b111
                } else {
                    unreachable!("SpiSlMiso {pin_nb}")
                }
            }
            PinMode::SpiSlCs => {
                if pin_nb == 21 {
                    0b111
                } else {
                    unreachable!("SpiSlCs {pin_nb}")
                }
            }
            PinMode::JtagTrst => {
                if pin_nb == 22 {
                    0b011
                } else {
                    unreachable!("JtagTrst {pin_nb}")
                }
            }
            PinMode::JtagRtck => match pin_nb {
                6 => 0b010,
                23 => 0b011,
                arg => unreachable!("JtagRtck {arg}"),
            },
            PinMode::JtagTdo => match pin_nb {
                5 => 0b010,
                24 => 0b011,
                arg => unreachable!("JtagTdo {arg}"),
            },
            PinMode::JtagTck => match pin_nb {
                13 => 0b010,
                25 => 0b011,
                arg => unreachable!("JtagTck {arg}"),
            },
            PinMode::JtagTdi => match pin_nb {
                4 => 0b010,
                26 => 0b011,
                arg => unreachable!("JtagTdi {arg}"),
            },
            PinMode::JtagTms => match pin_nb {
                12 => 0b010,
                27 => 0b011,
                arg => unreachable!("JtagTms {arg}"),
            },
        }
    }
}

register_bitfields! {
    u32,

    /// GPIO Function Select 0
    GPFSEL0 [
        FSEL0 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sda0 = 0b100,
            Sa5 = 0b101,
        ],

        FSEL1 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Scl0 = 0b100,
            Sa4 = 0b101,
        ],

        FSEL2 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sda1 = 0b100,
            Sa3 = 0b101,
        ],

        FSEL3 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Scl1 = 0b100,
            Sa2 = 0b101,
        ],

        FSEL4 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk0 = 0b100,
            Sa1 = 0b101,
            ArmTdi = 0b010,
        ],

        FSEL5 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk1 = 0b100,
            Sa0 = 0b101,
            ArmTdo = 0b010,
        ],

        FSEL6 OFFSET(18) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk2 = 0b100,
            Smi_SoeN_Se = 0b101,
            ArmRtck = 0b010,
        ],

        FSEL7 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Cs1 = 0b100,
            Smi_SweN_SrwN = 0b101,
        ],

        FSEL8 OFFSET(24) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Cs0 = 0b100,
            Sd0 = 0b101,
        ],

        FSEL9 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Miso = 0b100,
            Sd1 = 0b101,
        ],
    ],

    /// GPIO Function Select 1
    GPFSEL1 [
        FSEL10 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Mosi = 0b100,
            Sd2 = 0b101,
        ],

        FSEL11 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Sclk = 0b100,
            Sd3 = 0b101,
        ],

        FSEL12 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Pwm0 = 0b100,
            Sd4 = 0b101,
            ArmTms = 0b010,
        ],

        FSEL13 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Pwm1 = 0b100,
            Sd5 = 0b101,
            ArmTck = 0b010,
        ],

        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Uart0Txd = 0b100,
            Sd6 = 0b101,
            Uart1Txd = 0b010,
        ],

        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Uart0Rxd = 0b100,
            Sd7 = 0b101,
            Uart1Rxd = 0b010,

        ],

        FSEL16 OFFSET(18) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd8 = 0b101,
            Uart0Cts = 0b111,
            Spi1Cs2 = 0b011,
            Uart1Cts = 0b010,
        ],

        FSEL17 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd9 = 0b101,
            Uart0Rts = 0b111,
            Spi1Cs1 = 0b011,
            Uart1Rts = 0b010,
        ],

        FSEL18 OFFSET(24) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            PcmClk = 0b100,
            Sd10 = 0b101,
            BscSlaveSda_SpiSlaveMosi = 0b111,
            Spi1Cs0 = 0b011,
            Pwm0 = 0b010,
        ],

        FSEL19 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            PcmFs = 0b100,
            Sd11 = 0b101,
            BscSlaveScl_SpiSlaveSclk = 0b111,
            Spi1Miso = 0b011,
            Pwm1 = 0b010,
        ],
    ],

    /// GPIO Function Select 2
    GPFSEL2 [
        FSEL20 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            PcmDin = 0b100,
            Sd12 = 0b101,
            SpiSlaveMiso = 0b111,
            Spi1Mosi = 0b011,
            GpClk0 = 0b010,
        ],

        FSEL21 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            PcmDout = 0b100,
            Sd13 = 0b101,
            SpiSlaveCs = 0b111,
            Spi1Sclk = 0b011,
            GpClk1 = 0b010,
        ],

        FSEL22 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd14 = 0b101,
            Sd1Clk = 0b111,
            ArmTrst = 0b011,
        ],

        FSEL23 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd15 = 0b101,
            Sd1Cmd = 0b111,
            ArmRtck = 0b011,
        ],

        FSEL24 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd16 = 0b101,
            Sd1Dat0 = 0b111,
            ArmTdo = 0b011,
        ],

        FSEL25 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd17 = 0b101,
            Sd1Dat1 = 0b111,
            ArmTck = 0b011,
        ],

        FSEL26 OFFSET(18) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd1Dat2 = 0b111,
            ArmTdi = 0b011,
        ],

        FSEL27 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sd1Dat3 = 0b111,
            ArmTms = 0b011,
        ],

        FSEL28 OFFSET(24) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sda0 = 0b100,
            Sa5 = 0b101,
            PcmClk = 0b110,
        ],

        FSEL29 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Scl0 = 0b100,
            Sa4 = 0b101,
            PcmFs = 0b110,
        ],
    ],

    /// GPIO Function Select 3
    GPFSEL3 [
        FSEL30 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sa3 = 0b101,
            PcmDin = 0b110,
            Uart0Cts = 0b111,
            Uart1Cts = 0b010,
        ],

        FSEL31 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sa2 = 0b101,
            PcmDout = 0b110,
            Uart0Rts = 0b111,
            Uart1Rts = 0b010,
        ],

        FSEL32 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk0 = 0b100,
            Sa1 = 0b101,
            Uart0Txd = 0b111,
            Uart1Txd = 0b010,
        ],

        FSEL33 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Sa0 = 0b101,
            Uart0Rxd = 0b111,
            Uart1Rxd = 0b010,
        ],

        FSEL34 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk0 = 0b100,
            Smi_SoeN_Se = 0b101,
        ],

        FSEL35 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Cs1 = 0b100,
            Smi_SweN_SrwN = 0b101,
        ],

        FSEL36 OFFSET(18) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Cs0 = 0b100,
            Sd0 = 0b101,
            Uart0Txd = 0b110,
        ],

        FSEL37 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Miso = 0b100,
            Sd1 = 0b101,
            Uart0Rxd = 0b110,
        ],

        FSEL38 OFFSET(24) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Mosi = 0b100,
            Sd2 = 0b101,
            Uart0Rts = 0b110,
        ],

        FSEL39 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Spi0Sclk = 0b100,
            Sd3 = 0b101,
            Uart0Cts = 0b110,
        ],
    ],

    /// GPIO Function Select 4
    GPFSEL4 [
        FSEL40 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Pwm0 = 0b100,
            Sd4 = 0b101,
            Spi2Miso = 0b011,
            Uart1Txd = 0b010,
        ],

        FSEL41 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Pwm1 = 0b100,
            Sd5 = 0b101,
            Spi2Mosi = 0b011,
            Uart1Rxd = 0b010,
        ],

        FSEL42 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk1 = 0b100,
            Sd6 = 0b101,
            Spi2Sclk = 0b011,
            Uart1Rts = 0b010,
        ],

        FSEL43 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk2 = 0b100,
            Sd7 = 0b101,
            Spi2Cs0 = 0b011,
            Uart1Cts = 0b010,
        ],

        FSEL44 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            GpClk1 = 0b100,
            Sda0 = 0b101,
            Sda1 = 0b110,
            Spi2Cs1 = 0b011,
        ],

        FSEL45 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            Pwm1 = 0b100,
            Scl0 = 0b101,
            Scl1 = 0b110,
            Spi2Cs2 = 0b011,
        ],

        FSEL46 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

        FSEL47 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

        FSEL48 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

        FSEL49 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],
    ],

    /// GPIO Function Select 5
    GPFSEL5 [
        FSEL50 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

        FSEL51 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

        FSEL52 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

        FSEL53 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
        ],

    ],

    // Set a pin
    GPSET0 [ SET OFFSET(0) NUMBITS(32) ],
    GPSET1 [ SET OFFSET(0) NUMBITS(22) ],

    // Clear a pin
    GPCLR0 [ CLR OFFSET(0) NUMBITS(32) ],
    GPCLR1 [ CLR OFFSET(0) NUMBITS(22) ],

    // Pin level
    GPLEV0 [ LEV OFFSET(0) NUMBITS(32) ],
    GPLEV1 [ LEV OFFSET(0) NUMBITS(22) ],

    // Event Detect Status
    GPEDS0 [ EDS OFFSET(0) NUMBITS(32) ],
    GPEDS1 [ EDS OFFSET(0) NUMBITS(22) ],

    // Rising Edge Detect Enable
    GPREN0 [ REN OFFSET(0) NUMBITS(32) ],
    GPREN1 [ REN OFFSET(0) NUMBITS(22) ],

    // Falling Edge Detect Enable
    GPFEN0 [ FEN OFFSET(0) NUMBITS(32) ],
    GPFEN1 [ FEN OFFSET(0) NUMBITS(22) ],

    // Falling Edge Detect Enable
    GPHEN0 [ HEN OFFSET(0) NUMBITS(32) ],
    GPHEN1 [ HEN OFFSET(0) NUMBITS(22) ],

    // Falling Edge Detect Enable
    GPLEN0 [ LEN OFFSET(0) NUMBITS(32) ],
    GPLEN1 [ LEN OFFSET(0) NUMBITS(22) ],

    // Falling Edge Detect Enable
    GPAREN0 [ AREN OFFSET(0) NUMBITS(32) ],
    GPAREN1 [ AREN OFFSET(0) NUMBITS(22) ],

    // Falling Edge Detect Enable
    GPAFEN0 [ AFEN OFFSET(0) NUMBITS(32) ],
    GPAFEN1 [ AFEN OFFSET(0) NUMBITS(22) ],

    /// GPIO Pull-up/down Register
    GPPUD [
        /// Controls the actuation of the internal pull-up/down control line to ALL the GPIO pins.
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [ PUDCLK0 OFFSET(0) NUMBITS(32) ],
    GPPUDCLK1 [ PUDCLK1 OFFSET(0) NUMBITS(32) ],
}

register_structs! {
    #[allow(non_snake_case)]
    RegisterBlock {
        (0x00 => GPFSEL0: ReadWrite<u32, GPFSEL0::Register>),
        (0x04 => GPFSEL1: ReadWrite<u32, GPFSEL1::Register>),
        (0x08 => GPFSEL2: ReadWrite<u32, GPFSEL2::Register>),
        (0x0C => GPFSEL3: ReadWrite<u32, GPFSEL3::Register>),
        (0x10 => GPFSEL4: ReadWrite<u32, GPFSEL4::Register>),
        (0x14 => GPFSEL5: ReadWrite<u32, GPFSEL4::Register>),
        (0x18 => _reserved1),
        (0x1C => GPSET0: WriteOnly<u32, GPSET0::Register>),
        (0x20 => GPSET1: WriteOnly<u32, GPSET1::Register>),
        (0x24 => _reserved2),
        (0x28 => GPCLR0: WriteOnly<u32, GPCLR0::Register>),
        (0x2C => GPCLR1: WriteOnly<u32, GPCLR1::Register>),
        (0x30 => _reserved3),
        (0x34 => GPLEV0: ReadOnly<u32, GPLEV0::Register>),
        (0x38 => GPLEV1: ReadOnly<u32, GPLEV1::Register>),
        (0x3C => _reserved4),
        (0x40 => GPEDS0: ReadWrite<u32, GPEDS0::Register>),
        (0x44 => GPEDS1: ReadWrite<u32, GPEDS1::Register>),
        (0x48 => _reserved5),
        (0x4C => GPREN0: ReadWrite<u32, GPREN0::Register>),
        (0x50 => GPREN1: ReadWrite<u32, GPREN1::Register>),
        (0x54 => _reserved6),
        (0x58 => GPFEN0: ReadWrite<u32, GPFEN0::Register>),
        (0x5C => GPFEN1: ReadWrite<u32, GPFEN1::Register>),
        (0x60 => _reserved7),
        (0x64 => GPHEN0: ReadWrite<u32, GPHEN0::Register>),
        (0x68 => GPHEN1: ReadWrite<u32, GPHEN1::Register>),
        (0x6C => _reserved8),
        (0x70 => GPLEN0: ReadWrite<u32, GPLEN0::Register>),
        (0x74 => GPLEN1: ReadWrite<u32, GPLEN1::Register>),
        (0x78 => _reserved9),
        (0x7C => GPAREN0: ReadWrite<u32, GPAREN0::Register>),
        (0x80 => GPAREN1: ReadWrite<u32, GPAREN1::Register>),
        (0x84 => _reserved10),
        (0x88 => GPAFEN0: ReadWrite<u32, GPAFEN0::Register>),
        (0x8C => GPAFEN1: ReadWrite<u32, GPAFEN1::Register>),
        (0x90 => _reserved11),
        (0x94 => GPPUD: ReadWrite<u32, GPPUD::Register>),
        (0x98 => GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>),
        (0x9C => GPPUDCLK1: ReadWrite<u32, GPPUDCLK1::Register>),
        (0xA0 => _reserved12),
        (0xB4 => @END),
    }
}

/// Abstraction for the associated MMIO registers.
type GpioRegisters = MMIODerefWrapper<RegisterBlock>;
