use shim::const_assert_size;

struct FileHeader {
    magic_number: [u8; 4],
    bit_width: u8,
    endianness: u8,
    ei_version: u8,
    os_abi: u8,
    abi_version: u8,
    padding: [u8; 7],
    object_type: u8,
    machine: u16,
    e_version: u8,
    entry_point: u64,
    program_head: u64,
    section_head: u64,
    flags: u32,
    header_size: u16,
    program_head_size: u16,
    program_head_entries: u16,
    section_head_size: u16,
    section_head_entries: u16,
    names_section_entry: u16,
}

const_assert_size!(0x40);

