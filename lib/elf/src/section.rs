use crate::section::SectionHeaderType::Unknown;

#[derive(Debug)]
pub(crate) enum SectionHeaderType {
    ShtNull = 0x0,
    ShtProgbits = 0x1,
    ShtSymtab = 0x2,
    ShtStrtab = 0x3,
    ShtRela = 0x4,
    ShtHash = 0x5,
    ShtDynamic = 0x6,
    ShtNote = 0x7,
    ShtNobits = 0x8,
    ShtRel = 0x9,
    ShtShlib = 0x0A,
    ShtDynsym = 0x0B,
    ShtInitArray = 0x0E,
    ShtFiniArray = 0x0F,
    ShtPreinitArray = 0x10,
    ShtGroup = 0x11,
    ShtSymtabShndx = 0x12,
    ShtNum = 0x13,
    ShtLoos = 0x60000000,
    Unknown,
}

impl From<u32> for SectionHeaderType {
    fn from(value: u32) -> Self {
        match value {
            0x0 => SectionHeaderType::ShtNull,
            0x1 => SectionHeaderType::ShtProgbits,
            0x2 => SectionHeaderType::ShtSymtab,
            0x3 => SectionHeaderType::ShtStrtab,
            0x4 => SectionHeaderType::ShtRela,
            0x5 => SectionHeaderType::ShtHash,
            0x6 => SectionHeaderType::ShtDynamic,
            0x7 => SectionHeaderType::ShtNote,
            0x8 => SectionHeaderType::ShtNobits,
            0x9 => SectionHeaderType::ShtRel,
            0x0A => SectionHeaderType::ShtShlib,
            0x0B => SectionHeaderType::ShtDynsym,
            0x0E => SectionHeaderType::ShtInitArray,
            0x0F => SectionHeaderType::ShtFiniArray,
            0x10 => SectionHeaderType::ShtPreinitArray,
            0x11 => SectionHeaderType::ShtGroup,
            0x12 => SectionHeaderType::ShtSymtabShndx,
            0x13 => SectionHeaderType::ShtNum,
            0x60000000 => SectionHeaderType::ShtLoos,
            _ => SectionHeaderType::Unknown,
        }
    }
}

#[derive(Debug)]
pub(crate) enum SectionAttributes {
    ShfWrite = 0x1,
    ShfAlloc = 0x2,
    ShfExecinstr = 0x4,
    ShfMerge = 0x10,
    ShfStrings = 0x20,
    ShfInfoLink = 0x40,
    ShfLinkOrder = 0x80,
    ShfOsNonconforming = 0x100,
    ShfGroup = 0x200,
    ShfTls = 0x400,
    ShfMaskos = 0x0FF00000,
    ShfMaskproc = 0xF0000000,
    ShfOrdered = 0x4000000,
    ShfExclude = 0x8000000,
}

#[derive(Debug)]
pub struct SectionHeader {
    pub(crate) sh_name: u32,
    pub(crate) sh_type: SectionHeaderType,
    pub(crate) sh_flags: u64,
    pub(crate) sh_addr: u64,
    pub(crate) sh_offset: u64,
    pub(crate) sh_size: u64,
    pub(crate) sh_link: u32,
    pub(crate) sh_info: u32,
    pub(crate) sh_addralign: u64,
    pub(crate) sh_entsize: u64,
}