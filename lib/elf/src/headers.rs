use shim::const_assert_size;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct FileHeader {
    pub(crate) magic_number: [u8; 4],
    pub(crate) bit_width: u8,
    pub(crate) endianness: u8,
    pub(crate) ei_version: u8,
    pub(crate) os_abi: u8,
    pub(crate) abi_version: u8,
    pub(crate) padding: [u8; 7],
    pub(crate) object_type: u8,
    pub(crate) machine: u16,
    pub(crate) e_version: u8,
    pub(crate) entry_point: u64,
    pub(crate) program_table_offset: u64,
    pub(crate) section_table_offset: u64,
    pub(crate) flags: u32,
    pub(crate) header_size: u16,
    pub(crate) program_entry_size: u16,
    pub(crate) program_table_entries: u16,
    pub(crate) section_entry_size: u16,
    pub(crate) section_table_entries: u16,
    pub(crate) names_section_entry: u16,
}

const_assert_size!(FileHeader, 0x40);

#[repr(C)]
#[derive(Debug)]
pub(crate) struct ProgramHeader {
    segment_type: u32,
    flags: u32,
    segment_offset: u64,
    virtual_address: u64,
    physical_segment: u64,
    file_size: u64,
    memory_size: u64,
    alignment: u64,
}

const_assert_size!(ProgramHeader, 0x38);

#[repr(C)]
#[derive(Debug)]
pub(crate) struct SectionHeader {
    section_name: u32,
    header_type: u32,
    section_attributes: u64,
    virtual_address: u64,
    image_offset: u64,
    image_size: u64,
    section_index: u32,
    section_info: u32,
    alignment: u64,
    entry_size: u64,
}

const_assert_size!(SectionHeader, 0x40);
