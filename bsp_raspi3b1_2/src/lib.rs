#![allow(dead_code, unused_variables)]
#![no_std]

#[cfg(not(feature = "builder"))]
mod boot;
mod cpu;
mod mailboxes;
mod memory;
mod sync;

pub use cpu::spin_for_cycles;

pub mod console;
pub mod drivers;
pub mod errors;
pub mod init;
pub mod screen;

#[cfg(feature = "builder")]
pub const LINKER_SCRIPT: &str = include_str!("kernel.ld");
