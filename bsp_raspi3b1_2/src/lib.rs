#![allow(dead_code, unused_variables)]
#![no_std]

#[cfg(not(feature = "builder"))]
mod boot;
mod cpu;
mod mailboxes;
mod memory;
mod sync;

pub use cpu::spin_for_cycles;

use crate::drivers::PinMode;

pub mod console;
pub mod drivers;
pub mod errors;
pub mod init;
pub mod screen;

const MAX_CHAINLOAD_BINARY_SIZE : u32 = u32::MAX;    // TODO    To define

#[cfg(feature = "builder")]
pub const LINKER_SCRIPT: &str = include_str!("kernel.ld");

pub fn chainloader_binary_load() -> ! {
    let gpio = &drivers::GPIO;
    gpio.configure(&[(21, PinMode::Output)]);
    gpio.set_pin(21);

    let uart = &drivers::UART;
    assert!(uart.init.lock(|i| *i), "Cannot chainload: UART not initialized");
    dbg!("Flush");
    uart.flush();
    dbg!("Clear RX");
    uart.clear_rx();
    dbg!("Writing init");
    uart.write("333");    // INIT
    gpio.clear_pin(21);

    dbg!("Waiting for size");
    // Read the binary's size.
    let mut size: u32 = u32::from(uart.read_char(true).unwrap() as u8);
    size |= u32::from(uart.read_char(true).unwrap() as u8) << 8;
    size |= u32::from(uart.read_char(true).unwrap() as u8) << 16;
    size |= u32::from(uart.read_char(true).unwrap() as u8) << 24;
    assert!(size < MAX_CHAINLOAD_BINARY_SIZE);
    gpio.set_pin(21);

    uart.write("OK");
    gpio.clear_pin(21);

    let kernel_addr: *mut u8 = memory::BOARD_DEFAULT_LOAD_ADDRESS as *mut u8;
    // Read the kernel byte by byte.
    for i in 0..size {
        unsafe {
            core::ptr::write_volatile(
                kernel_addr.offset(i as isize),
                uart.read_char(true).unwrap() as u8
            );
        }
     }
    println!("[ML] Loaded! Executing the payload now\n");
    uart.flush();
    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
    kernel()
}
