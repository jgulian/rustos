use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use shim::io::{Read, Seek, Write};
use format::Format;
use crate::metadata::{Metadata, Timestamp};

pub(crate) enum DirectoryAttribute {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20,
    LongFileName = 0b1111,
}

#[derive(Copy, Clone, Format)]
pub(crate)  struct RegularDirectoryEntry {
    name: [u8; 8],
    extension: [u8; 3],
    attributes: u8,
    nt_reserved: u8,
    created_time_tenth: u8,
    created_time: u32,
    last_access: u16,
    first_cluster_high: u16,
    last_modification: u16,
    first_cluster_low: u16,
    file_size: u32,
}

#[derive(Copy, Clone, Format)]
pub(crate) struct LongFileNameEntry {
    order: u8,
    name_one: [u8; 10],
    attributes: u8,
    dir_type: u8,
    checksum: u8,
    name_two: [u8; 12],
    first_cluster_low: u16,
    name_three: [u8; 4],
}

impl LongFileNameEntry {
    pub(crate) fn new(order: u8, name: &[u8]) -> Self {
        // TODO: fill in these values
        let mut result = Self {
            order,
            name_one: [0u8; 10],
            attributes: DirectoryAttribute::LongFileName as u8,
            dir_type: 0,
            checksum: 0,
            name_two: [0u8; 12],
            first_cluster_low: 0,
            name_three: [0u8; 4],
        };

        let mut buffer = [0u8; 26];
        (&mut buffer[..name.len()]).copy_from_slice(name);
        let vec_u16: Vec<u16> = String::from_utf8_lossy(&buffer).encode_utf16().collect();
        let vec_u8: Vec<u8> = vec_u16.iter()
            .flat_map(|d| [(d & 0xFF) as u8, (d >> 8 & 0xFF) as u8])
            .collect();
        let slice = vec_u8.as_slice();

        result.name_one.copy_from_slice(&slice[..10]);
        result.name_two.copy_from_slice(&slice[10..22]);
        result.name_three.copy_from_slice(&slice[22..26]);

        result
    }
}

pub(crate) enum DirectoryEntry {
    Empty,
    Regular(RegularDirectoryEntry),
    LongFileName(LongFileNameEntry),
    EmptyAndOver,
}

impl format::Format for DirectoryEntry {
    fn load_readable<T: Read>(stream: &mut T) -> shim::io::Result<Self> {
        let mut slice = [0u8; 32];
        stream.read_exact(&mut slice)?;
        match slice[11] {
            0 => Ok(Self::EmptyAndOver),
            0xe5 => Ok(Self::Empty),
            0b1111 => {
                Ok(Self::LongFileName(LongFileNameEntry::load_slice(&slice)?))
            }
            _ => Ok(Self::Regular(RegularDirectoryEntry::load_slice(&slice)?)),
        }
    }

    fn load_readable_seekable<T: Read + Seek>(stream: &mut T) -> shim::io::Result<Self> {
        Self::load_readable(stream)
    }

    fn save_writable<T: Write>(&self, stream: &mut T) -> shim::io::Result<()> {
        match self {
            DirectoryEntry::Regular(regular) => regular.save_writable(stream),
            DirectoryEntry::LongFileName(long_file_name) => long_file_name.save_writable(stream),
            DirectoryEntry::Empty => {
                let mut slice = [0u8; 32];
                slice[11] = 0xe5;
                stream.write_all(&slice)
            }
            DirectoryEntry::EmptyAndOver => {
                let slice = [0u8; 32];
                stream.write_all(&slice)
            }
        }
    }

    fn save_writable_seekable<T: Write + Seek>(&self, stream: &mut T) -> shim::io::Result<()> {
        self.save_writable(stream)
    }
}

pub(crate) fn parse_name(long_file_names: &mut Vec<LongFileNameEntry>, regular: &RegularDirectoryEntry) -> String {
    if long_file_name.is_empty() {
        let mut result = String::new();
        for c in regular.name.iter() {
            if *c == b' ' || *c == 0 {
                break;
            }
            result.push(*c as char);
        }
        let mut has_extension = false;
        for c in regular.extension.iter() {
            if *c == b' ' || *c == 0 {
                break;
            }
            if !has_extension {
                result.push('.');
                has_extension = true;
            }
            result.push(*c as char)
        }
        result
    } else {
        long_file_names.sort_by(|a, b|
            (a.order & 0b11111).cmp(&(b.order & 0b11111)));

        let mut bytes = Vec::<u8>::new();
        for long_file_name in long_file_names.iter() {
            bytes.extend_from_slice(&long_file_name.name_one);
            bytes.extend_from_slice(&long_file_name.name_two);
            bytes.extend_from_slice(&long_file_name.name_three);
        }

        let mut chars = Vec::<u16>::new();
        for byte_pair in bytes.chunks(2) {
            let char = (byte_pair[1] as u16) << 8 | byte_pair[0] as u16;
            if char == 0 || char == 0xFFFF {
                break;
            }
            chars.push(char);
        }

        String::from_utf16_lossy(chars.as_slice())
    }
}

pub(crate) fn parse_entry(regular_entry: &RegularDirectoryEntry) -> (u32, Metadata, Option<u32>) {
    let starting_cluster = ((regular_dir.first_cluster_high as u32) << 16) |
        (regular_dir.first_cluster_low as u32);

    let metadata = Metadata {
        attributes: regular_entry.attributes,
        created: Default::default(),
        last_access: Default::default(),
        last_modification: Default::default(),
    };

    if regular_entry.attributes & (DirectoryAttribute::Directory as u8) > 0 {
        (starting_cluster, metadata, None)
    } else {
        (starting_cluster, metadata, Some(regular_entry.file_size))
    }
}