#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use crate::device::BlockDevice;
use crate::master_boot_record::PartitionEntry;
use crate::path::{Component, Path};
use shim::io;

type BoxedFile = Box<dyn File>;
type BoxedDirectory = Box<dyn Directory>;

pub trait Timestamp {
    fn year(&self) -> usize;
    fn month(&self) -> u8;
    fn day(&self) -> u8;
    fn hour(&self) -> u8;
    fn minute(&self) -> u8;
    fn second(&self) -> u8;
}

pub trait Metadata {
    fn read_only(&self) -> bool;

    fn hidden(&self) -> bool;

    fn created(&self) -> Box<dyn Timestamp>;

    fn accessed(&self) -> Box<dyn Timestamp>;

    fn modified(&self) -> Box<dyn Timestamp>;
}

pub trait File: io::Seek + io::Read + io::Write + Send + Sync {
    fn duplicate(&mut self) -> io::Result<Box<dyn File>>;
}

pub trait Directory: Send + Sync {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry>;
    fn create_file(&mut self, name: &str) -> io::Result<()>;
    fn create_directory(&mut self, name: &str) -> io::Result<()>;
    fn remove(&mut self, name: &str) -> io::Result<()>;
    fn list(&mut self) -> io::Result<Vec<String>>;
    fn metadata(&mut self) -> io::Result<Box<dyn Metadata>>;

    fn exists(&mut self, name: &str) -> io::Result<bool> {
        match self.open_entry(name) {
            Ok(_) => Ok(true),
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

pub enum Entry {
    File(BoxedFile),
    Directory(BoxedDirectory),
}

impl Entry {
    pub fn into_file(self) -> io::Result<BoxedFile> {
        match self {
            Entry::File(file) => Ok(file),
            Entry::Directory(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    pub fn into_directory(self) -> io::Result<BoxedDirectory> {
        match self {
            Entry::File(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
            Entry::Directory(directory) => Ok(directory),
        }
    }

    pub fn as_file(&mut self) -> io::Result<&mut BoxedFile> {
        match self {
            Entry::File(file) => Ok(file),
            Entry::Directory(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }

    pub fn as_directory(&mut self) -> io::Result<&mut BoxedDirectory> {
        match self {
            Entry::File(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
            Entry::Directory(directory) => Ok(directory),
        }
    }
}

pub trait Filesystem: Send + Sync {
    fn root(&mut self) -> io::Result<BoxedDirectory>;

    fn open(&mut self, path: &Path) -> io::Result<Entry> {
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                Component::Root => {
                    components.push(Entry::Directory(self.root()?));
                }
                Component::Parent => {
                    components
                        .pop()
                        .ok_or(io::Error::from(io::ErrorKind::NotFound))?;
                }
                Component::Current => {}
                Component::Child(child) => {
                    let new_entry = components
                        .last_mut()
                        .ok_or(io::Error::from(io::ErrorKind::NotFound))
                        .map(|entry| {
                            let a = entry.as_directory()?;
                            a.open_entry(child.as_str())
                        })??;
                    components.push(new_entry);
                }
            }
        }

        components
            .pop()
            .map(Ok)
            .unwrap_or(Err(io::Error::from(io::ErrorKind::NotFound)))
    }

    fn format(
        device: &mut dyn BlockDevice,
        partition: &mut PartitionEntry,
        sector_size: usize,
    ) -> io::Result<()>
    where
        Self: Sized;
}
