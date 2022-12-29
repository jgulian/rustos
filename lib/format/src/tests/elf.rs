use crate::Format;
use std::io::{Read, Seek, SeekFrom, Result, Write, Cursor};

#[derive(format_derive::Format, Debug)]
struct ElfTableInfo {
    #[endianness(little)] entry_size: u16,
    #[endianness(little)] entry_count: u16,
}

#[derive(format_derive::Format, Debug)]
struct ElfFileHeader {
    #[endianness(little)] magic_number: u32,
    class: u8,
    endianness: u8,
    current_version: u8,
    os_abi: u8,
    abi_version: u8,
    #[padding(7)]
    #[endianness(little)] object_type: u16,
    #[endianness(little)] machine: u16,
    #[endianness(little)] original_version: u64,
    #[endianness(little)] program_header_table_offset: u64,
    #[endianness(little)] section_header_table_offset: u64,
    #[endianness(little)] flags: u32,
    #[endianness(little)] header_size: u16,
    program_header_table: ElfTableInfo,
    section_header_table: ElfTableInfo,
    #[endianness(little)] section_name_index: u16,
}

#[test]
fn elf() {
    let sample_header = [0o312u8, 0o376u8, 0o272u8, 0o276u8, 0o000u8, 0o000u8, 0o000u8, 0o002u8, 0o001u8, 0o000u8, 0o000u8, 0o007u8, 0o000u8, 0o000u8, 0o000u8, 0o003,
        0o000u8, 0o000u8, 0o100u8, 0o000u8, 0o000u8, 0o011u8, 0o304u8, 0o160u8, 0o000u8, 0o000u8, 0o000u8, 0o016u8, 0o001u8, 0o000u8, 0o000u8, 0o014,
        0o200u8, 0o000u8, 0o000u8, 0o002u8, 0o000u8, 0o012u8, 0o100u8, 0o000u8, 0o000u8, 0o011u8, 0o375u8, 0o360u8, 0o000u8, 0o000u8, 0o000u8, 0o016,
        0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000u8, 0o000];

    let mut cursor = Cursor::new(sample_header);
    let header = ElfFileHeader::load_readable(&mut cursor);
    println!("header: {:?}", header);
}