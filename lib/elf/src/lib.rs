#![no_std]

mod headers;
mod elf;

pub(crate) extern crate alloc;

pub use elf::Elf;