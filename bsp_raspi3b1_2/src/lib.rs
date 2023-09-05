#![allow(dead_code, unused_variables)]
#![no_std]

#[cfg(not(feature = "builder"))]
mod boot;
#[cfg(not(feature = "builder"))]
mod cpu;
#[cfg(not(feature = "builder"))]
mod mailboxes;
#[cfg(not(feature = "builder"))]
mod memory;

#[cfg(not(feature = "builder"))]
pub mod console;
#[cfg(not(feature = "builder"))]
pub mod drivers;
#[cfg(not(feature = "builder"))]
pub mod errors;
#[cfg(not(feature = "builder"))]
pub mod gpio;
#[cfg(not(feature = "builder"))]
pub mod init;
#[cfg(not(feature = "builder"))]
pub mod screen;

#[cfg(feature = "builder")]
pub const LINKER_SCRIPT: &str = include_str!("kernel.ld");
