#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::collections::BTreeMap;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::collections::BTreeMap;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::sync::Arc;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use crate::device::BlockDevice;
use crate::filesystem::{Directory, Entry, File, Filesystem, Metadata};
use crate::master_boot_record::PartitionEntry;
use crate::path::Path;
use shim::io;
use shim::io::{Read, Seek, SeekFrom, Write};
use sync::Mutex;

pub enum EntryType {
    File,
    Directory,
}

pub trait PseudoDirectory: Send {
    fn read(&mut self, offset: &Path, buf: &mut [u8]) -> io::Result<usize>;
    fn write(&mut self, offset: &Path, buf: &[u8]) -> io::Result<usize>;
    fn list(&mut self, offset: &Path) -> io::Result<Vec<String>>;
    fn entry_type(&mut self, offset: &Path) -> Option<EntryType>;
}

pub struct PseudoDirectoryCollection(BTreeMap<Path, Box<dyn PseudoDirectory>>);

impl PseudoDirectoryCollection {
    pub fn new(data: BTreeMap<Path, Box<dyn PseudoDirectory>>) -> Self {
        Self(data)
    }

    fn find(&mut self, needle: &Path) -> Option<(Path, &mut Box<dyn PseudoDirectory>)> {
        self.0.iter_mut().find_map(|(path, directory)| {
            needle.relative_from(path).map(|offset| (offset, directory))
        })
    }

    fn list(&mut self, needle: &Path) -> io::Result<Vec<String>> {
        let mut result = Vec::new();
        self.0
            .iter_mut()
            .try_for_each(|(path, directory)| -> io::Result<()> {
                if let Some(offset) = needle.relative_from(path) {
                    result.extend(directory.list(&offset)?);
                }
                Ok(())
            })?;
        Ok(result)
    }
}

pub struct PseudoFilesystem<M: Mutex<PseudoDirectoryCollection>>(Arc<M>);

impl<M: Mutex<PseudoDirectoryCollection> + 'static> PseudoFilesystem<M> {
    pub fn new(directories: PseudoDirectoryCollection) -> Self {
        Self(Arc::new(M::new(directories)))
    }
}

impl<M: Mutex<PseudoDirectoryCollection> + 'static> Filesystem for PseudoFilesystem<M> {
    fn root(&mut self) -> io::Result<Box<dyn Directory>> {
        Ok(Box::new(PseudoFilesystemLocation(
            Path::default(),
            self.0.clone(),
        )))
    }

    fn format(_: &mut dyn BlockDevice, _: &mut PartitionEntry, _: usize) -> io::Result<()>
    where
        Self: Sized,
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "can not format a pseudo filesystem",
        ))
    }
}

pub struct PseudoFilesystemLocation<M: Mutex<PseudoDirectoryCollection>>(Path, Arc<M>);

impl<M: Mutex<PseudoDirectoryCollection> + 'static> Directory for PseudoFilesystemLocation<M> {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry> {
        let mut path = self.0.clone();
        path.join_str(name)?;
        let entry = self
            .1
            .lock(|directory_collection| {
                let directory = directory_collection.find(&path)?.1;
                directory.entry_type(&path)
            })
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "poisoned lock in pseudo filesystem",
                )
            })?
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "entry not found in pseudo filesystem",
            ))?;

        let location = PseudoFilesystemLocation(path, self.1.clone());

        Ok(match entry {
            EntryType::File => Entry::File(Box::new(location)),
            EntryType::Directory => Entry::Directory(Box::new(location)),
        })
    }

    fn create_file(&mut self, _name: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "can not mutate a pseudo filesystem",
        ))
    }

    fn create_directory(&mut self, _name: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "can not mutate a pseudo filesystem",
        ))
    }

    fn remove(&mut self, _name: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "can not mutate a pseudo filesystem",
        ))
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        self.1
            .lock(|directory_collection| directory_collection.list(&self.0))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "poisoned lock in pseudo filesystem",
                )
            })?
    }

    fn metadata(&mut self) -> io::Result<Box<dyn Metadata>> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "metadata unsupported on pseudo filesystem",
        ))
    }
}

impl<M: Mutex<PseudoDirectoryCollection>> Seek for PseudoFilesystemLocation<M> {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "seek unsupported on pseudo filesystem",
        ))
    }
}

impl<M: Mutex<PseudoDirectoryCollection>> Read for PseudoFilesystemLocation<M> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.1
            .lock(|directory_collection| {
                let directory = directory_collection
                    .find(&self.0)
                    .ok_or(io::Error::new(
                        io::ErrorKind::NotFound,
                        "entry not found in pseudo filesystem",
                    ))?
                    .1;
                directory.read(&self.0, buf)
            })
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "poisoned lock in pseudo filesystem",
                )
            })?
    }
}

impl<M: Mutex<PseudoDirectoryCollection>> Write for PseudoFilesystemLocation<M> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.1
            .lock(|directory_collection| {
                let directory = directory_collection
                    .find(&self.0)
                    .ok_or(io::Error::new(
                        io::ErrorKind::NotFound,
                        "entry not found in pseudo filesystem",
                    ))?
                    .1;
                directory.write(&self.0, buf)
            })
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "poisoned lock in pseudo filesystem",
                )
            })?
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "flush unsupported on pseudo filesystem",
        ))
    }
}

impl<M: Mutex<PseudoDirectoryCollection> + 'static> File for PseudoFilesystemLocation<M> {
    fn duplicate(&mut self) -> io::Result<Box<dyn File>> {
        Ok(Box::new(self.clone()))
    }
}

impl<M: Mutex<PseudoDirectoryCollection>> Clone for PseudoFilesystemLocation<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}
