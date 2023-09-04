// First to be called
//    Initialize default values everywhere
//    Disable all devices by default
//    Set all pins as input by default
//    Init IRQ controller and Timer
//    Init allocator
//    Init GPU
pub fn init_bsp() {
    todo!();
}

fn init_allocator() {
    todo!();
}

fn disable_all_devices() {
    todo!();
}

fn init_irq_controller() {
    todo!();
}

fn init_timer() {
    todo!();
}

//     Initialization of devices

use crate::drivers::{SpiDriver, UartDriver};

pub fn init_uart() -> UartDriver {
    todo!();
}

pub fn init_spi(nb: u8) -> SpiDriver {
    todo!();
}
