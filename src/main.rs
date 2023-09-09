#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::drivers::PinMode;
use bsp_raspi3b1_2::errors::handle_panic;
use bsp_raspi3b1_2::{println, spin_for_cycles, chainloader_binary_load};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.panic_led_on();
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    let uart = &bsp_raspi3b1_2::drivers::UART;
    uart.configure(14, 15);
    chainloader_binary_load();
}
