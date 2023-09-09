use core::fmt::Write;

use crate::sync::NullLock;

pub static CONSOLE: Console = Console::init();

pub struct Console(NullLock<ConsoleInner>);

impl Console {
    pub const fn init() -> Console {
        Console(NullLock::new(ConsoleInner::init()))
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result {
        self.0.lock(|console| {
            console.write_fmt(args)
        })
    }
}

struct ConsoleInner;

impl ConsoleInner {
    pub const fn init() -> ConsoleInner {
        ConsoleInner {}
    }
}

impl core::fmt::Write for ConsoleInner {
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        let uart = &crate::drivers::UART;
        uart.write_char(c);
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

pub fn _print_raw_str_nl(s: &str) {
    let addr = 0x3F20_1000 as *mut u8;
    for c in s.chars() {
        if c.is_ascii() {
            unsafe { core::ptr::write_volatile(addr, c as u8); }
        }
    }
    unsafe { core::ptr::write_volatile(addr, '\n' as u8); }
}

pub fn _print_raw_fmt_nl(args: core::fmt::Arguments) {
    _print_raw_str_nl(args.as_str().unwrap())
}

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

#[macro_export]
macro_rules! dbg {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::console::_print_raw_fmt_nl(format_args!($($arg)*));
    })
}
