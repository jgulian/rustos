#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

pub(crate) mod cluster;
pub(crate) mod fat;
pub(crate) mod ebpb;
pub(crate) mod file;
pub(crate) mod metadata;
pub(crate) mod chain;
pub(crate) mod vfat;

