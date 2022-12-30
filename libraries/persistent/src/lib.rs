#![no_std]

#![feature(decl_macro)]

extern crate alloc;

pub use self::devices::{BlockDevice, CharDevice};
pub use self::vfs::VirtualFileSystem;

pub mod devices;
pub mod vfs;
pub mod fs2;
pub mod path;

#[cfg(test)]
mod tests;

