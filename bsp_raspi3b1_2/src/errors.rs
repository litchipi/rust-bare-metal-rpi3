use core::panic::PanicInfo;

use crate::cpu::wait_forever;

pub fn handle_panic(info: &PanicInfo) -> ! {
    wait_forever();
}
