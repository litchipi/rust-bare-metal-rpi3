#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::drivers::PinMode;
use bsp_raspi3b1_2::errors::handle_panic;
use bsp_raspi3b1_2::{println, spin_for_cycles};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic: {info:?}");
    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.panic_led_on();
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    // FIXME    Stuck somewhere waiting for UART
    //    Create debug macros (to write to raw memory) and debug this
    //    Once UART messages prints to emulation, try on raspberry pi
    let uart = &bsp_raspi3b1_2::drivers::UART;
    uart.configure(14, 15);
    println!("Done");

    println!("Hello world!");

    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.configure(&[(21, PinMode::Output)]);
    spin_for_cycles(100_000);

    loop {
        println!("Led ON");
        gpios.set_pin(21);
        spin_for_cycles(1_000_000);

        println!("Led OFF");
        gpios.clear_pin(21);
        spin_for_cycles(1_000_000);
    }
}
