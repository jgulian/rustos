#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

pub mod device;
pub mod error;
pub mod filesystem;
pub mod mbr;
pub mod path;
pub mod vfs;
pub mod partition;

#[cfg(test)]
mod tests;