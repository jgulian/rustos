use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem::transmute;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;

use crate::traits;
use crate::traits::Dummy;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Error, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};
use crate::vfat::vfat::ChainOffset;

enum DirectoryAttribute {
    ReadOnly = 0x01,
    Hidden = 0x02,
    System = 0x04,
    VolumeId = 0x08,
    Directory = 0x10,
    Archive = 0x20,
    LongFileName = 0b1111,
}

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub first_cluster: Cluster,
    pub name: String,
    pub metadata: Metadata,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    name: [u8; 8],
    extension: [u8; 3],
    attributes: u8,
    __nt_reserved: u8,
    created_time_tenth: u8,
    created_time: Timestamp,
    last_access: Date,
    first_cluster_high: u16,
    last_modification: Timestamp,
    first_cluster_low: u16,
    file_size: u32,
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    order: u8,
    name_one: [u8; 10],
    attributes: u8,
    dir_type: u8,
    checksum: u8,
    name_two: [u8; 12],
    first_cluster_low: u16,
    name_three: [u8; 4],
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    named: u8,
    __reserved_one: [u8; 10],
    attributes: u8,
    __reserved_two: [u8; 20],
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        use traits::{Dir, Entry};
        let name = name.as_ref().to_str()
            .ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;

        for entry in self.entries()? {
            if str::eq_ignore_ascii_case(entry.name(), name) {
                return Ok(entry);
            }
        }

        Err(io::Error::from(io::ErrorKind::NotFound))
    }
}

impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>;
    type Iter = DirIter<HANDLE>;

    fn entries(&self) -> io::Result<Self::Iter> {
        let mut data: Vec<u8> = Vec::new();
        self.vfat.lock(|file_system|
            file_system.read_chain(self.first_cluster, &mut data)
        )?;

        Ok(DirIter {
            vfat: self.vfat.clone(),
            data: unsafe { data.cast() },
            i: 0,
            done: false,
        })
    }
}

pub struct DirIter<HANDLE: VFatHandle> {
    vfat: HANDLE,
    data: Vec<VFatDirEntry>,
    i: usize,
    done: bool,
}

impl<HANDLE: VFatHandle> Iterator for DirIter<HANDLE> {
    type Item = Entry<HANDLE>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut long_file_names = Vec::<VFatLfnDirEntry>::new();

        loop {
            if self.done || self.i == self.data.len() {
                return None;
            }

            let entry: &VFatDirEntry = &self.data[self.i];
            if unsafe { entry.unknown.named } == 0 {
                self.done = true;
                return None;
            }

            if unsafe { entry.unknown.named } == 0xe5 {
                self.i += 1;
                long_file_names.clear();
                continue;
            }

            self.i += 1;

            let (regular, long_file_name) = unsafe {
                if entry.unknown.attributes == DirectoryAttribute::LongFileName as u8 {
                    (None, Some(entry.long_filename))
                } else {
                    (Some(entry.regular), None)
                }
            };

            if long_file_name.is_some() {
                long_file_names.push(long_file_name.unwrap());
                continue;
            }

            let regular_dir = regular.unwrap();
            let mut name = String::new();

            let name = if long_file_names.is_empty() {
                let mut result = String::new();
                let mut found_dot = false;
                for c in regular_dir.name.iter() {
                    if *c == b' ' || *c == 0 {
                        break;
                    }
                    result.push(*c as char);
                }
                let mut has_extension = false;
                for c in regular_dir.extension.iter() {
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
                parse_name(&mut long_file_names)
            };

            if name.len() == 0 {
                panic!("entry must have name");
            }

            let starting_cluster = ((regular_dir.first_cluster_high as u32) << 16) |
                (regular_dir.first_cluster_low as u32);

            let metadata = Metadata {
                attributes: regular_dir.attributes,
                created: regular_dir.created_time,
                last_access: Timestamp { date: regular_dir.last_access, time: Default::default() },
                last_modification: regular_dir.last_modification,
            };

            let entry = if regular_dir.attributes & (DirectoryAttribute::Directory as u8) > 0 {
                Entry::Dir(Dir {
                    vfat: self.vfat.clone(),
                    first_cluster: Cluster::from(starting_cluster),
                    name,
                    metadata,
                })
            } else {
                Entry::File(File {
                    vfat: self.vfat.clone(),
                    name,
                    metadata,
                    file_size: regular_dir.file_size,
                    offset: ChainOffset::new(Cluster::from(starting_cluster)),
                })
            };

            return Some(entry);
        }
    }
}

fn parse_name(long_file_names: &mut Vec<VFatLfnDirEntry>) -> String {
    long_file_names.sort_by(|a, b| (a.order & 0b11111).cmp(&(b.order & 0b11111)));

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