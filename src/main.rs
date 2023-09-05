
#![no_main]
#![no_std]

use core::panic::PanicInfo;

use bsp_raspi3b1_2::errors::handle_panic;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    handle_panic(info);
}
