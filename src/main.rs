#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::drivers::PinMode;
use bsp_raspi3b1_2::errors::handle_panic;
use bsp_raspi3b1_2::{println, spin_for_cycles};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    println!("Hello world!");

    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.configure(&[(14, PinMode::Output)]);

    loop {
        println!("Led ON");
        gpios.set_pin(14);
        spin_for_cycles(120_000_000);

        println!("Led OFF");
        gpios.clear_pin(14);
        spin_for_cycles(120_000_000);
    }
}
