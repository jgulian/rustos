use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};

use log::info;

use shim::{io, ioerr, newioerr};
use shim::io::ErrorKind::NotFound;

use crate::Metadata;
use crate::path::{Component, Path};

pub trait Metadata2 {}

// For char devices, their seek just gives a NotSeekable error
pub trait File2: io::Seek + io::Read + io::Write + Drop {
    fn duplicate(&mut self) -> io::Result<Box<dyn File2>>;
}

pub trait Directory2 {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry2>;
    fn create_file(&mut self, name: &str) -> io::Result<()>;
    fn create_directory(&mut self, name: &str) -> io::Result<()>;
    fn remove(&mut self, name: &str) -> io::Result<()>;

    fn list(&mut self) -> io::Result<Vec<String>>;
    fn metadata(&mut self, name: &str) -> io::Result<Box<dyn Metadata2>>;

    fn exists(&mut self, name: &str) -> io::Result<bool> {
        let result = self.open_entry(name);
        Ok(match result {
            Ok(_) => true,
            Err(ref e) => {
                e.kind() == io::ErrorKind::NotFound
            }
        })
    }
}

pub trait FileSystem2 {
    fn root(&mut self) -> io::Result<Box<dyn Directory2>>;

    fn open(&mut self, path: &Path) -> io::Result<Entry2> {
        path.simplify()?.components().iter()
            .fold(ioerr!(NotFound), |wrapped_entry, component| {
                match component {
                    Component::Root => {
                        match wrapped_entry {
                            Ok(entry) => {
                                return ioerr!(InvalidFilename);
                            }
                            Err(ref err) => {
                                if err.kind() != NotFound {
                                    return ioerr!(InvalidFilename);
                                }
                            }
                        }

                        Ok(Entry2::Directory(self.root()?))
                    }
                    Component::Current | Component::Parent => {
                        ioerr!(InvalidFilename)
                    }
                    Component::Child(child) => {
                        match wrapped_entry? {
                            Entry2::File(_) => ioerr!(InvalidFilename),
                            Entry2::Directory(mut dir) => {
                                dir.open_entry(child.as_str())
                            }
                        }
                    }
                }
            })
    }

    fn copy_entry(&mut self, source: &Path, destination: &Path) -> io::Result<()>;

    fn move_entry(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        //self.copy_entry(source, destination)?;
        //match self.open(source.parent().ok_or(newioerr!(InvalidFilename))?)?.borrow_mut() {
        //    Entry2::File(_) => ioerr!(InvalidFilename),
        //    Entry2::Directory(directory) => {
        //        let file_name = source.file_name()
        //            .ok_or(newioerr!(InvalidFilename))?.to_str()
        //            .ok_or(newioerr!(InvalidFilename))?;
        //        directory.remove(file_name)
        //    }
        //}
        todo!()
    }
}

pub enum Entry2 {
    File(Box<dyn File2>),
    Directory(Box<dyn Directory2>),
}

impl Entry2 {
    pub fn into_file(self) -> Option<Box<dyn File2>> {
        match self {
            Entry2::File(file) => Some(file),
            Entry2::Directory(_) => None,
        }
    }

    pub fn into_directory(self) -> Option<Box<dyn Directory2>> {
        match self {
            Entry2::File(_) => None,
            Entry2::Directory(directory) => Some(directory),
        }
    }
}