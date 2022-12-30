use alloc::string::String;
use alloc::vec::Vec;

use log::info;

use filesystem;

use shim::const_assert_size;
use shim::io;
use shim::io::{Read, Seek, SeekFrom, Write};

use crate::util::VecExt;
use crate::vfat::{Date, Metadata, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};
use crate::vfat::vfat::Chain;

enum DirectoryAttribute {
    _ReadOnly = 0x01,
    _Hidden = 0x02,
    _System = 0x04,
    _VolumeId = 0x08,
    Directory = 0x10,
    _Archive = 0x20,
    LongFileName = 0b1111,
}

#[derive(Debug, Clone)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub name: String,
    pub metadata: Metadata,
    pub(crate) chain: Chain<HANDLE>,
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

//FIXME: use u16?
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

impl VFatLfnDirEntry {
    fn new(order: u8, name: &[u8]) -> Self {
        // TODO: fill in these values
        let mut result = VFatLfnDirEntry {
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
    pub fn find(&mut self, name: &str) -> io::Result<Entry<HANDLE>> {
        use filesystem::{Dir, Entry};

        for entry in self.entries()? {
            if str::eq_ignore_ascii_case(entry.name(), name) {
                return Ok(entry);
            }
        }

        Err(io::Error::from(io::ErrorKind::NotFound))
    }
}

impl<HANDLE: VFatHandle> filesystem::Dir for Dir<HANDLE> {
    type Entry = Entry<HANDLE>;
    type Iter = DirIter<HANDLE>;

    fn entries(&mut self) -> io::Result<Self::Iter> {
        let mut data: Vec<u8> = Vec::new();
        let _read = self.chain.read_to_end(&mut data)?;

        Ok(DirIter {
            vfat: self.vfat.clone(),
            data: unsafe { data.cast() },
            i: 0,
            done: false,
        })
    }

    fn append(&mut self, entry: Self::Entry) -> io::Result<()> {
        use filesystem::Entry;

        let mut bytes: Vec<u8> = Vec::new();
        self.chain.seek(SeekFrom::Start(0))?;
        self.chain.read_to_end(&mut bytes)?;
        let mut entries: Vec<VFatDirEntry> = unsafe { bytes.cast() };

        let mut i = 0;
        while unsafe { entries[i].unknown.named } != 0 {
            i += 1;
        }

        info!("old bytes {}", (i * core::mem::size_of::<VFatDirEntry>()));

        while i < entries.len() {
            entries.remove(i);
        }

        for lfn in serialize_lfns(entry.name()) {
            entries.push(VFatDirEntry { long_filename: lfn });
        }

        let mut regular_dir_entry = VFatRegularDirEntry {
            name: [0; 8],
            extension: [0; 3],
            attributes: 0,
            __nt_reserved: 0,
            created_time_tenth: 0,
            created_time: Default::default(),
            last_access: Default::default(),
            first_cluster_high: 0,
            last_modification: Default::default(),
            first_cluster_low: 0,
            file_size: 0,
        };

        for (i, c) in entry.name().as_bytes().iter().take(8).enumerate() {
            regular_dir_entry.name[i] = c.to_ascii_uppercase();
        }
        info!("actual name {:?}", regular_dir_entry.name);

        entries.push(VFatDirEntry { regular: regular_dir_entry });

        bytes = unsafe { entries.cast() };
        info!("here 3 new bytes {} {}", bytes.len(), bytes[core::mem::size_of::<VFatDirEntry>() * 3]);
        //TODO: we should be able to get away with seeking to new data.
        self.chain.seek(SeekFrom::Start(0))?;
        self.chain.write_all(bytes.as_slice())?;
        Ok(())
    }

    fn remove(&mut self, _entry: Self::Entry) -> io::Result<()> {
        unimplemented!("not implemented")
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

            //FIXME: This needs to be sorted by position
            let name = if long_file_names.is_empty() {
                let mut result = String::new();
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

            //TODO: check the checksum?

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

            let first_cluster = Cluster::from(starting_cluster);
            let chain = {
                let chain = Chain::new_from_cluster(self.vfat.clone(), first_cluster);
                if chain.is_err() {
                    panic!("unable to find cluster start");
                }

                chain.unwrap()
            };

            let entry = if regular_dir.attributes & (DirectoryAttribute::Directory as u8) > 0 {
                Entry::Dir(Dir {
                    vfat: self.vfat.clone(),
                    name,
                    metadata,
                    chain,
                })
            } else {
                Entry::File(File {
                    name,
                    metadata,
                    file_size: regular_dir.file_size,
                    chain,
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

fn serialize_lfns(name: &str) -> Vec<VFatLfnDirEntry> {
    name.as_bytes().chunks(26).enumerate().map(|(i, chunk)| {
        VFatLfnDirEntry::new(i as u8 + 1, chunk)
    }).collect()
}