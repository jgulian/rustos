#![cfg_attr(feature = "no_std", no_std)]

#![feature(decl_macro)]

extern crate alloc;

pub use self::devices::{BlockDevice, CharDevice};
pub use self::metadata::{Metadata, Timestamp};
pub use self::traits::{Dir, Entry, File, FileSystem};
pub use self::vfs::VirtualFileSystem;

pub mod devices;
pub mod traits;
pub mod metadata;
pub mod vfs;
pub mod fs2;
pub mod mbr;
pub mod path;

#[cfg(test)]
mod tests;

