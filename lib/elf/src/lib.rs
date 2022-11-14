#![no_std]

mod file;
mod elf;
mod program;
pub(crate) mod reader;
mod util;
mod section;

pub(crate) extern crate alloc;

pub(crate) use file::{FileHeadIdentity, ObjectFileType, TargetMachineIsa, FileHeader};
pub(crate) use section::{SectionHeaderType, SectionAttributes};
pub(crate) use program::{ProgramHeaderType};

pub use elf::Elf;
pub use section::SectionHeader;
pub use program::ProgramHeader;
pub use util::File;