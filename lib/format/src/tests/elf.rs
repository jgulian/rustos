use crate::Format;
use std::io::{Cursor, Read, Result, Seek, SeekFrom, Write};

#[derive(format_derive::Format, Debug, Eq, PartialEq)]
struct ElfTableInfo {
    #[endianness(little)]
    entry_size: u16,
    #[endianness(little)]
    entry_count: u16,
}

#[derive(format_derive::Format, Debug, Eq, PartialEq)]
struct ElfFileHeader {
    magic_number: [u8; 4],
    class: u8,
    endianness: u8,
    current_version: u8,
    os_abi: u8,
    abi_version: u8,
    #[padding(7)]
    #[endianness(little)]
    object_type: u16,
    #[endianness(little)]
    machine: u16,
    #[endianness(little)]
    original_version: u32,
    #[endianness(little)]
    entry_point: u64,
    #[endianness(little)]
    program_header_table_offset: u64,
    #[endianness(little)]
    section_header_table_offset: u64,
    #[endianness(little)]
    flags: u32,
    #[endianness(little)]
    header_size: u16,
    program_header_table: ElfTableInfo,
    section_header_table: ElfTableInfo,
    #[endianness(little)]
    section_name_index: u16,
}

#[derive(format_derive::Format, Debug)]
struct ManyElfs {
    headers: [ElfFileHeader; 4],
    data: [u8; 256],
}

const SAMPLE_HEADER: [u8; 64] = [
    0x7f, 0x45, 0x4c, 0x46, 0x2, 0x1, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x0,
    0xb7, 0x0, 0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x0, 0x0, 0x0,
    0x0, 0x0, 0x0, 0x0, 0x88, 0x7f, 0x44, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x40, 0x0,
    0x38, 0x0, 0x4, 0x0, 0x40, 0x0, 0x12, 0x0, 0x10, 0x0,
];

// ELF Header:
//   Magic:   7f 45 4c 46 02 01 01 00 00 00 00 00 00 00 00 00
//   Class:                             ELF64
//   Data:                              2's complement, little endian
//   Version:                           1 (current)
//   OS/ABI:                            UNIX - System V
//   ABI Version:                       0
//   Type:                              EXEC (Executable file)
//   Machine:                           AArch64
//   Version:                           0x1
//   Entry point address:               0x80000
//   Start of program headers:          64 (bytes into file)
//   Start of section headers:          4489096 (bytes into file)
//   Flags:                             0x0
//   Size of this header:               64 (bytes)
//   Size of program headers:           56 (bytes)
//   Number of program headers:         4
//   Size of section headers:           64 (bytes)
//   Number of section headers:         18
//   Section header string table index: 16

#[test]
fn elf() {
    let mut cursor = Cursor::new(SAMPLE_HEADER);
    let header: ElfFileHeader = ElfFileHeader::load_readable(&mut cursor).expect("should be ok");

    //TODO: make these all enums
    assert_eq!(header.magic_number, [0x7F, 0x45, 0x4c, 0x46]);
    assert_eq!(header.class, 2);
    assert_eq!(header.endianness, 1);
    assert_eq!(header.current_version, 1);
    assert_eq!(header.os_abi, 0);
    assert_eq!(header.abi_version, 0);
    assert_eq!(header.object_type, 2);
    assert_eq!(header.machine, 0xB7);
    assert_eq!(header.original_version, 1);
    assert_eq!(header.entry_point, 0x80000);
    assert_eq!(header.program_header_table_offset, 64);
    assert_eq!(header.section_header_table_offset, 4489096);
    assert_eq!(header.flags, 0);
    assert_eq!(header.header_size, 64);
    assert_eq!(
        header.program_header_table,
        ElfTableInfo {
            entry_size: 56,
            entry_count: 4
        }
    );
    assert_eq!(
        header.section_header_table,
        ElfTableInfo {
            entry_size: 64,
            entry_count: 18
        }
    );
    assert_eq!(header.section_name_index, 16);
}

#[test]
fn many_elfs() {
    let sample_header = SAMPLE_HEADER.clone();
    let data: Vec<u8> = (0..256).map(|x| x as u8).collect();

    let mut among = sample_header.repeat(4);
    among.extend_from_slice(data.as_slice());

    let mut cursor = Cursor::new(among);
    let many_elfs = ManyElfs::load_readable(&mut cursor).expect("should be ok");

    let mut result = Cursor::new(vec![0u8; sample_header.len()]);
    many_elfs.headers[0]
        .save_writable_seekable(&mut result)
        .expect("should be ok");

    assert_eq!(many_elfs.headers[0], many_elfs.headers[1]);
    assert_eq!(many_elfs.headers[1], many_elfs.headers[2]);
    assert_eq!(many_elfs.headers[2], many_elfs.headers[3]);
    assert_eq!(result.get_ref().as_slice(), sample_header);
    assert_eq!(many_elfs.data, data.as_slice());
}
