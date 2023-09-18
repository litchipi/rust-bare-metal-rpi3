#![no_main]
#![no_std]

use core::{panic::PanicInfo, time::Duration};

use bsp_raspi3b1_2::drivers::gpio::PinMode;
use bsp_raspi3b1_2::timer::spin_for;
use bsp_raspi3b1_2::{errors::handle_panic, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.panic_led_on();
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    let mut big_addr: u64 = 16 * 1024 * 1024 * 1024;
    unsafe { core::ptr::read_volatile(big_addr as *mut u64) };

    let gpio = &bsp_raspi3b1_2::drivers::GPIO;
    gpio.configure(&[(21, PinMode::Output)]);
    loop {
        println!("LED ON");
        gpio.set_pin(21);
        spin_for(Duration::from_secs(1));

        println!("LED OFF");
        gpio.clear_pin(21);
        spin_for(Duration::from_secs(1));
    }
}
