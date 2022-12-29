use crate::Formatted;
use shim::io::{Read, Seek, SeekFrom, Result, Write};

#[derive(formatted_derive::Formatted)]
struct ElfTableInfo {
    #[endianness(big)] entry_size: u16,
    #[endianness(big)] entry_count: u16,
}

#[derive(formatted_derive::Formatted)]
struct ElfFileHeader {
    #[endianness(big)] magic_number: u32,
    class: u8,
    endianness: u8,
    current_version: u8,
    os_abi: u8,
    abi_version: u8,
    #[padding(7)]
    #[endianness(big)] object_type: u16,
    #[endianness(big)] machine: u16,
    #[endianness(big)] original_version: u64,
    #[endianness(big)] program_header_table_offset: u64,
    #[endianness(big)] section_header_table_offset: u64,
    #[endianness(big)] flags: u32,
    #[endianness(big)] header_size: u16,
    program_header_table: ElfTableInfo,
    section_header_table: ElfTableInfo,
    #[endianness(big)] section_name_index: u16,
}

#[test]
fn elf() {}