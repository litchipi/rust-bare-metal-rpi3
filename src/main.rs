#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::{drivers::gpio::PinMode, errors::handle_panic, println, spin_for_cycles};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.panic_led_on();
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    let gpio = &bsp_raspi3b1_2::drivers::GPIO;
    gpio.configure(&[(21, PinMode::Output)]);
    loop {
        println!("LED ON");
        gpio.set_pin(21);
        spin_for_cycles(2_000_000);

        println!("LED OFF");
        gpio.clear_pin(21);
        spin_for_cycles(500_000);
    }
}
