#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::sync::Arc;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use core::cell::RefCell;

use shim::{io, ioerr, newioerr};
use shim::io::{Error, ErrorKind};
use crate::device::BlockDevice;
use crate::filesystem::{Directory, Entry, File, Filesystem, Metadata};
use crate::mbr::PartitionEntry;

use crate::path::Path;

struct Mount {
    mount_point: Path,
    filesystem: Box<dyn Filesystem>,
}

//TODO: this is not thread safe
#[derive(Clone)]
struct Mounts(Arc<RefCell<Vec<Mount>>>);

pub struct VirtualFileSystem {
    mounts: Mounts,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        VirtualFileSystem {
            mounts: Mounts(Arc::new(RefCell::new(Vec::new()))),
        }
    }

    pub fn mount(&mut self, mount_point: Path, filesystem: Box<dyn Filesystem>) {
        self.mounts.0.as_ref().borrow_mut().push(Mount {
            mount_point,
            filesystem,
        })
    }
}

impl Filesystem for VirtualFileSystem {
    fn root(&mut self) -> io::Result<Box<dyn Directory>> {
        Ok(Box::new(VFSDirectory {
            path: Path::default(),
            mounts: self.mounts.clone(),
        }))
    }

    fn format(_: &mut dyn BlockDevice, _: &mut PartitionEntry, _: usize) -> io::Result<()> where Self: Sized {
        Err(Error::from(ErrorKind::Unsupported))
    }
}

struct VFSDirectory {
    path: Path,
    mounts: Mounts,
}

impl Directory for VFSDirectory {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry> {
        let mut new_path = self.path.clone();
        new_path.join_str(name);

        let mounts = self.mounts.clone();
        let mut mounts_borrow = mounts.0.as_ref().borrow_mut();
        mounts_borrow.iter_mut()
            .filter(|mount| mount.mount_point.starts_with(&self.path))
            .find_map(|mount| -> Option<Entry> {
                if mount.mount_point == self.path {
                    let mut thing = mount.filesystem.root().ok()?;
                    thing.open_entry(name).ok()
                } else if mount.mount_point.starts_with(&new_path) {
                    Some(Entry::Directory(Box::new(VFSDirectory {
                        path: new_path.clone(),
                        mounts: self.mounts.clone(),
                    })))
                } else {
                    None
                }
            }).ok_or(newioerr!(NotFound))
    }

    fn create_file(&mut self, name: &str) -> io::Result<Box<dyn File>> {
        self.mounts.0.as_ref().borrow_mut().iter_mut()
            .filter(|mount| mount.mount_point == self.path)
            .next().map(|mount| mount.filesystem.root()?.create_file(name))
            .unwrap_or(ioerr!(Unsupported))
    }

    fn create_directory(&mut self, _: &str) -> io::Result<Box<dyn Directory>> {
        ioerr!(Unsupported)
    }

    fn remove(&mut self, _: &str) -> io::Result<()> {
        ioerr!(Unsupported)
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        self.mounts.0.as_ref().borrow_mut().iter_mut()
            .try_fold(vec![], |mut result: Vec<String>, mount| {
                match self.path.relative_from(&mount.mount_point) {
                    Some(sub_path) => {
                        let entries = mount.filesystem.open(&sub_path)
                            .and_then(|entry| entry.into_directory())
                            .map(|mut directory| directory.list())
                            .unwrap_or(Ok(vec![]))?;
                        result.extend(entries.into_iter());
                    }
                    None => {}
                }

                Ok(result)
            })
    }

    fn metadata(&mut self, _: &str) -> io::Result<Box<dyn Metadata>> {
        ioerr!(Unsupported)
    }
}
