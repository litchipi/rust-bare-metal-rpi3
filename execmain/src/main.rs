#![no_main]
#![no_std]

use core::{panic::PanicInfo, time::Duration};

#[macro_use]
extern crate alloc;

use aarch64_cpu::registers::{CNTPCT_EL0, CNTP_CVAL_EL0};
use alloc::boxed::Box;
use bsp_raspi3b1_2::drivers::gpio::PinMode;
use bsp_raspi3b1_2::drivers::timer::spin_for;
use bsp_raspi3b1_2::init::{finish_init, init_bsp};
use bsp_raspi3b1_2::{wait_forever, print};
use bsp_raspi3b1_2::{errors::handle_panic, println};
use tock_registers::interfaces::Readable;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let gpios = &bsp_raspi3b1_2::drivers::GPIO;
    gpios.panic_led_on();
    handle_panic(info);
}

#[no_mangle]
pub fn _start_rust() -> ! {
    init_bsp();
    let context = Context::setup();
    println!("\n========================\n");
    println!("[*] Configuration done");
    finish_init();

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
            dur: Duration::from_millis(300),
            led: 21,
        }
    }

    pub fn main(self) -> ! {
        // let gpio = &bsp_raspi3b1_2::drivers::GPIO;
        // let mut test_vec = vec![];
        let mut n = 0u32;

        println!("[*] Setting up the timer");
        bsp_raspi3b1_2::drivers::TIMERS.set_timeout(
            Duration::from_secs(2),
            Some(Duration::from_secs(1)),
            Box::new(|| {
                println!("\n[*] Timer interrupt");
            }),
        );

        println!("[*] Starting loop");
        // wait_forever();
        loop {
            // print!(".");
            // aarch64_cpu::asm::wfi();
            // test_vec.push(n);
            // println!("[*] Test vec: {:?}", test_vec);
        //     println!("[*] LED ON");
        //     gpio.set_pin(self.led);
            // println!("{n}");
            spin_for(self.dur);

        //     println!("[*] LED OFF");
        //     gpio.clear_pin(self.led);
        //     spin_for(self.dur);
            n += 1;

            // let cs : u32 = unsafe {
            //     core::ptr::read_volatile((0x3F00_3000) as *const _)
            // };
            // let counter : u32 = unsafe {
            //     core::ptr::read_volatile((0x3F00_3004) as *const _)
            // };
            // let timers : u32 = unsafe {
            //     core::ptr::read_volatile((0x3F00_3010) as *const _)
            // };
            // println!("{cs}\t{counter}/{timers}");
            // if cs > 0 {
            //     panic!("End of test");
            // }

            let pending : u32 = unsafe {
                core::ptr::read_volatile((0x3F00_B200) as *const _)
            };
            let pending1 : u32 = unsafe {
                core::ptr::read_volatile((0x3F00_B204) as *const _)
            };
            let pending2 : u32 = unsafe {
                core::ptr::read_volatile((0x3F00_B208) as *const _)
            };
            let cnt = CNTPCT_EL0.get();
            let tot = CNTP_CVAL_EL0.get();
            println!("{pending} {pending1} {pending2}\t{cnt} / {tot}");
            if cnt > tot {
                panic!("End of test");
            }
        }
    }
}
