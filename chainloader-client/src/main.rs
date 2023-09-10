#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::drivers::gpio::PinMode;
use bsp_raspi3b1_2::errors::handle_panic;
use bsp_raspi3b1_2::{chainloader_binary_load, dbg, spin_for_cycles};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let uart = &bsp_raspi3b1_2::drivers::UART;
    uart.configure(14, 15);
    uart.write("KO");
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    let uart = &bsp_raspi3b1_2::drivers::UART;
    uart.configure(14, 15);
    chainloader_binary_load(uart);
    // loop {
    //     uart.write("3");
    //     spin_for_cycles(1_200_000);
    // }
}
