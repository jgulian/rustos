#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

pub mod cache;
pub mod device;
pub mod error;
pub mod filesystem;
pub mod master_boot_record;
pub mod path;
pub mod pseudo;
pub mod virtual_file_system;
pub mod partition;

#[cfg(not(feature = "no_std"))]
pub mod image;

#[cfg(test)]
mod tests;