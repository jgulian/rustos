use alloc::boxed::Box;
use shim::{const_assert_size, io};
use crate::File;
use crate::util::{BitWidth, Endianness};

const VALID_HEADER: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

#[derive(Debug)]
pub(crate) struct FileHeadIdentity([u8; 0x10]);

impl FileHeadIdentity {
    pub fn read(file: &mut Box<dyn File>) -> io::Result<Self> {
        let mut result = FileHeadIdentity([0; 0x10]);
        file.read_exact(&mut result.0)?;
        Ok(result)
    }

    pub fn is_valid_header(&self) -> bool {
        self.0[..4].eq(&VALID_HEADER)
    }

    pub fn bit_width(&self) -> BitWidth {
        match self.0[4] {
            1 => BitWidth::Bit32,
            2 => BitWidth::Bit64,
            _ => BitWidth::Unknown,
        }
    }

    pub fn endianness(&self) -> Endianness {
        match self.0[5] {
            1 => Endianness::Little,
            2 => Endianness::Big,
            _ => Endianness::Unknown,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ObjectFileType {
    EtNone = 0x00,
    EtRel = 0x01,
    EtExec = 0x02,
    EtDyn = 0x03,
    EtCore = 0x04,
    EtLoos = 0xFE00,
    EtHios = 0xFEFF,
    EtLoproc = 0xFF00,
    EtHiproc = 0xFFFF,
    Unknown,
}

impl From<u16> for ObjectFileType {
    fn from(value: u16) -> Self {
        match value {
            0x00 => ObjectFileType::EtNone,
            0x01 => ObjectFileType::EtRel,
            0x02 => ObjectFileType::EtExec,
            0x03 => ObjectFileType::EtDyn,
            0x04 => ObjectFileType::EtCore,
            0xFE00 => ObjectFileType::EtLoos,
            0xFEFF => ObjectFileType::EtHios,
            0xFF00 => ObjectFileType::EtLoproc,
            0xFFFF => ObjectFileType::EtHiproc,
            _ => ObjectFileType::Unknown,
        }
    }
}

#[derive(Debug)]
pub(crate) enum TargetMachineIsa {
    Aarch64 = 0xB7,
    Unknown,
}

impl From<u16> for TargetMachineIsa {
    fn from(value: u16) -> Self {
        match value {
            0xB7 => TargetMachineIsa::Aarch64,
            _ => TargetMachineIsa::Unknown,
        }
    }
}

#[derive(Debug)]
pub(crate) struct FileHeader {
    pub(crate) e_ident: FileHeadIdentity,
    pub(crate) e_type: ObjectFileType,
    pub(crate) e_machine: TargetMachineIsa,
    pub(crate) e_version: u32,
    pub(crate) e_entry: u64,
    pub(crate) e_phoff: u64,
    pub(crate) e_shoff: u64,
    pub(crate) e_flags: u32,
    pub(crate) e_ehsize: u16,
    pub(crate) e_phentsize: u16,
    pub(crate) e_phnum: u16,
    pub(crate) e_shentsize: u16,
    pub(crate) e_shnum: u16,
    pub(crate) e_shstrndx: u16,
}