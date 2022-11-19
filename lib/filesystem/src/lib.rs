#![no_std]

#![feature(decl_macro)]

extern crate alloc;

pub mod devices;
pub mod traits;
pub mod metadata;
mod vfs;

pub use self::devices::{BlockDevice, CharDevice};
pub use self::traits::{Dir, Entry, File, FileSystem};
pub use self::metadata::{Metadata, Timestamp};
pub use self::vfs::VirtualFileSystem;