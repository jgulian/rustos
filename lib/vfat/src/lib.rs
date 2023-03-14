#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

pub(crate) mod cluster;
pub(crate) mod directory;
pub(crate) mod entry;
pub(crate) mod fat;
pub(crate) mod bios_parameter_block;
pub(crate) mod error;
pub(crate) mod file;
pub(crate) mod metadata;
pub(crate) mod chain;
pub mod virtual_fat;

