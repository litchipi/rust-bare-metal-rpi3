use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::cpu::wait_forever;
use crate::println;

fn panic_prevent_reenter() {
    static PANIC_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

    if !PANIC_IN_PROGRESS.load(Ordering::Relaxed) {
        PANIC_IN_PROGRESS.store(true, Ordering::Relaxed);

        return;
    }

    wait_forever();
}

pub fn handle_panic(info: &PanicInfo) -> ! {
    println!("Kernel panic!");

    if let Some(payload) = info.payload().downcast_ref::<&str>() {
        println!("{}", payload);
    }

    if let Some(loc) = info.location() {
        println!(
            "On file {} (line {},  col {})",
            loc.file(),
            loc.line(),
            loc.column()
        );
    }

    println!("{info:?}");

    wait_forever();
}
