use core::fmt::Write;

pub struct Console;

impl Console {
    pub fn init() -> Console {
        Console {}
    }

    // pub fn write(&self, msg: String) {
    //     todo!();
    // }

    // pub fn read(&self) -> Option<String> {
    //     todo!();
    // }
}

impl core::fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe {
                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
            }
        }
        Ok(())
    }
}

pub fn _print(args: core::fmt::Arguments) {
    Console::init().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::console::_print(format_args!($($arg)*));
        $crate::console::_print(format_args!("\n"));
    })
}
