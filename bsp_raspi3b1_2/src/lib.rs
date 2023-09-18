#![allow(dead_code, unused_variables)]
#![no_std]

#[cfg(not(feature = "builder"))]
mod boot;
mod cpu;
mod mailboxes;
mod memory;
mod sync;

pub mod console;
pub mod drivers;
pub mod errors;
pub mod init;
pub mod irq;
pub mod screen;
pub mod timer;

const MAX_CHAINLOAD_BINARY_SIZE: u32 = u32::MAX; // TODO    To define

#[cfg(feature = "builder")]
pub const LINKER_SCRIPT: &str = include_str!("kernel.ld");

pub fn chainloader_binary_load(uart: &drivers::uart::UartDriver) -> ! {
    assert!(
        uart.init.lock(|i| *i),
        "Cannot chainload: UART not initialized"
    );
    uart.flush();
    uart.clear_rx();
    loop {
        uart.write("333"); // INIT
        let c = uart.read_char(true).unwrap();
        if c == 'u' {
            break;
        }
    }

    // Read the binary's size.
    let mut size: u32 = u32::from(uart.read_char(true).unwrap() as u8);
    size |= u32::from(uart.read_char(true).unwrap() as u8) << 8;
    size |= u32::from(uart.read_char(true).unwrap() as u8) << 16;
    size |= u32::from(uart.read_char(true).unwrap() as u8) << 24;
    assert!(size < MAX_CHAINLOAD_BINARY_SIZE);

    uart.write("OK");

    let kernel_addr: *mut u8 = memory::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;

    // Read the kernel byte by byte.
    for i in 0..size {
        unsafe {
            core::ptr::write_volatile(
                kernel_addr.offset(i.try_into().unwrap()),
                uart.read_byte(true).unwrap(),
            );
        }
    }
    uart.write("OK");
    uart.flush();
    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
    kernel()
}
