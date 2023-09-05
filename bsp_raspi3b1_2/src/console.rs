use core::fmt::Write;

use crate::sync::NullLock;

pub struct ConsoleInner;

impl ConsoleInner {
    pub const fn init() -> ConsoleInner {
        ConsoleInner {}
    }
}

impl core::fmt::Write for ConsoleInner {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        unsafe {
            core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
        }
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.write_char('\r')?;
            }

            self.write_char(c)?;
        }
        Ok(())
    }
}

pub struct Console(NullLock<ConsoleInner>);

impl Console {
    pub const fn init() -> Console {
        Console(NullLock::new(ConsoleInner::init()))
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result {
        self.0.lock(|inner| inner.write_fmt(args))
    }
}

static CONSOLE: Console = Console::init();

pub fn _print(args: core::fmt::Arguments) {
    CONSOLE.write_fmt(args).unwrap();
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
