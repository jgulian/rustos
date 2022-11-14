use alloc::boxed::Box;
use shim::{const_assert_size, io};
use shim::io::{Seek, SeekFrom};
use crate::File;
use crate::reader::ElfReader;

#[derive(Debug)]
pub enum ProgramHeaderType {
    PtNull = 0,
    PtLoad = 1,
    PtDynamic = 2,
    PtInterp = 3,
    PtNote = 4,
    PtShlib = 5,
    PtPhdr = 6,
    PtTls = 7,
    PtLoos = 0x60000000,
    PtGnuEhFrame = 0x6474e550,
    PtGnuStack = 0x6474e551,
    PtHios = 0x6FFFFFFF,
    PtLoproc = 0x70000000,
    PtHiproc = 0x7FFFFFFF,
    PtUnknown,
}

impl From<u32> for ProgramHeaderType {
    fn from(value: u32) -> Self {
        match value {
            0 => ProgramHeaderType::PtNull,
            1 => ProgramHeaderType::PtLoad,
            2 => ProgramHeaderType::PtDynamic,
            3 => ProgramHeaderType::PtInterp,
            4 => ProgramHeaderType::PtNote,
            5 => ProgramHeaderType::PtShlib,
            6 => ProgramHeaderType::PtPhdr,
            7 => ProgramHeaderType::PtTls,
            0x60000000 => ProgramHeaderType::PtLoos,
            0x6474e550 => ProgramHeaderType::PtGnuEhFrame,
            0x6474e551 => ProgramHeaderType::PtGnuStack,
            0x6FFFFFFF => ProgramHeaderType::PtHios,
            0x70000000 => ProgramHeaderType::PtLoproc,
            0x7FFFFFFF => ProgramHeaderType::PtHiproc,
            _ => ProgramHeaderType::PtUnknown,
        }
    }
}

#[derive(Debug)]
pub struct ProgramHeader {
    pub(crate) p_type: ProgramHeaderType,
    pub(crate) p_flags: u32,
    pub(crate) p_offset: u64,
    pub(crate) p_vaddr: u64,
    pub(crate) p_paddr: u64,
    pub(crate) p_filesz: u64,
    pub(crate) p_memsz: u64,
    pub(crate) p_align: u64,
}
