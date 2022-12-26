use formatted_derive::Formatted;

#[derive(Formatted)]
struct ElfTableInfo {
    #[entry(endianness = big)] entry_size: u16,
    #[entry(endianness = big)] entry_count: u16,
}

#[derive(Formatted)]
struct ElfFileHeader {
    #[entry(endianness = big)] magic_number: u32,
    class: u8,
    endianness: u8,
    current_version: u8,
    os_abi: u8,
    abi_version: u8,
    #[entry(pad = 7, endianness = big)] object_type: u16,
    #[entry(endianness = big)] machine: u16,
    #[entry(endianness = big)] original_version: u64,
    #[entry(endianness = big)] program_header_table_offset: u64,
    #[entry(endianness = big)] section_header_table_offset: u64,
    #[entry(endianness = big)] flags: u32,
    #[entry(endianness = big)] header_size: u16,
    program_header_table: ElfTableInfo,
    section_header_table: ElfTableInfo,
    #[entry(endianness = big)] section_name_index: u16,
}

#[test]
fn elf() {

}