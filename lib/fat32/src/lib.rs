#![feature(decl_macro)]
#![cfg_attr(feature = "no_std", no_std)]

#[macro_use]
extern crate alloc;
#[cfg(not(feature = "no_std"))]
extern crate core;

pub use crate::mbr::*;

#[cfg(not(target_endian = "little"))]
compile_error!("only little endian platforms supported");

mod mbr;
#[cfg(test)]
mod tests;
mod util;

pub mod vfat;

