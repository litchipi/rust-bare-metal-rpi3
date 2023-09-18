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
    bsp_raspi3b1_2::init();
    let context = Context::setup();
    println!("\n========================\n");
    println!("[*] Configuration done");
    bsp_raspi3b1_2::finish_init();

    println!("[*] Initialization finished");
    context.main()
}

pub struct Context {
    dur: Duration,
    led: usize,
}

impl Context {
    pub fn setup() -> Context {
        let uart = &bsp_raspi3b1_2::drivers::UART;
        uart.configure(14, 15);

        let gpio = &bsp_raspi3b1_2::drivers::GPIO;
        gpio.configure(&[(21, PinMode::Output)]);

        Context {
            dur: Duration::from_secs(2),
            led: 21,
        }
    }

    pub fn main(self) -> ! {
        let gpio = &bsp_raspi3b1_2::drivers::GPIO;
        println!("[*] Starting loop");
        loop {
            println!("[*] LED ON");
            gpio.set_pin(self.led);
            spin_for(self.dur);

            println!("[*] LED OFF");
            gpio.clear_pin(self.led);
            spin_for(self.dur);
        }
    }
}
