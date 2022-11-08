#![no_std]

#![feature(decl_macro)]

extern crate alloc;

pub mod block_device;
pub mod dummy;
pub mod fs;
pub mod metadata;

pub use self::block_device::BlockDevice;
pub use self::dummy::Dummy;
pub use self::fs::{Dir, Entry, File, FileSystem};
pub use self::metadata::{Metadata, Timestamp};
