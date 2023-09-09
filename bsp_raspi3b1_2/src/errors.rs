use core::panic::PanicInfo;

use crate::cpu::wait_forever;
use crate::println;

#[derive(Debug)]
pub enum Errcode {}

pub fn handle_panic(info: &PanicInfo) -> ! {
    println!("Kernel panic ! {info}");
    wait_forever();
}
