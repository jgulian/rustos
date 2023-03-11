#![cfg_attr(feature = "no_std", no_std)]

#![feature(decl_macro)]

#[cfg(feature = "no_std")]
extern crate alloc;

pub mod device;
pub mod filesystem;
pub mod mbr;
pub mod path;
pub mod vfs;

#[cfg(test)]
mod tests;