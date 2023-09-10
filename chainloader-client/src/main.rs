#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::errors::handle_panic;
use bsp_raspi3b1_2::{chainloader_binary_load, dbg};
use bsp_raspi3b1_2::drivers::PinMode;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let gpio = &bsp_raspi3b1_2::drivers::GPIO;
    gpio.configure(&[
        (21, PinMode::Output),
    ]);
    gpio.set_pin(21);
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    let uart = &bsp_raspi3b1_2::drivers::UART;
    uart.configure(14, 15);
    dbg!("Starting chainloading...");
    chainloader_binary_load();
}
