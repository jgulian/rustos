use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};

use log::info;

use shim::{io, ioerr, newioerr};

use crate::fs2;
use crate::fs2::{Directory2, Entry2, FileSystem2, Metadata2};
use crate::path::{Component, Path};

struct Mount {
    mount_point: Path,
    filesystem: Box<dyn FileSystem2>,
}

//TODO: this is not thread safe
#[derive(Clone)]
struct Mounts(Arc<RefCell<Vec::<Mount>>>);

pub struct VirtualFileSystem {
    mounts: Mounts,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        VirtualFileSystem {
            mounts: Mounts(Arc::new(RefCell::new(Vec::new()))),
        }
    }

    pub fn mount(&mut self, mount_point: Path, filesystem: Box<dyn FileSystem2>) {
        self.mounts.0.as_ref().borrow_mut().push(Mount {
            mount_point,
            filesystem,
        })
    }
}

impl FileSystem2 for VirtualFileSystem {
    fn root(&mut self) -> io::Result<Box<dyn Directory2>> {
        Ok(Box::new(VFSDirectory {
            path: Path::root(),
            mounts: self.mounts.clone(),
        }))
    }

    fn copy_entry(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        todo!()
    }
}

struct VFSDirectory {
    path: Path,
    mounts: Mounts,
}

impl Directory2 for VFSDirectory {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry2> {
        let mut new_path = self.path.clone();
        new_path.append_child(name.to_string());

        let mounts = self.mounts.clone();
        let result = mounts.0.as_ref().borrow_mut().iter_mut()
            .filter(|mount| mount.mount_point.starts_with(&self.path))
            .find_map(|mount| -> Option<Entry2> {
                if mount.mount_point == self.path {
                    let mut thing = mount.filesystem.root().ok()?;
                    thing.open_entry(name).ok()
                } else if mount.mount_point.starts_with(&new_path) {
                    Some(Entry2::Directory(Box::new(VFSDirectory {
                        path: new_path.clone(),
                        mounts: self.mounts.clone(),
                    })))
                } else {
                    None
                }
            }).ok_or(newioerr!(NotFound));
        result
    }

    fn create_file(&mut self, name: &str) -> io::Result<()> {
        self.mounts.0.as_ref().borrow_mut().iter_mut()
            .filter(|mount| mount.mount_point == self.path)
            .next().map(|mount| mount.filesystem.root()?.create_file(name))
            .unwrap_or(ioerr!(Unsupported))
    }

    fn create_directory(&mut self, name: &str) -> io::Result<()> {
        ioerr!(Unsupported)
    }

    fn remove(&mut self, _: &str) -> io::Result<()> {
        ioerr!(Unsupported)
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        self.mounts.0.as_ref().borrow_mut().iter_mut()
            .filter(|mount| mount.mount_point.starts_with(&self.path))
            .fold(Ok(Vec::new()), |wrapped_result, mount| {
                let mut result = wrapped_result?;

                if let Some(component) = mount.mount_point.at(self.path.len() + 1) {
                    match component {
                        Component::Child(child) => {
                            result.push(child)
                        }
                        _ => {}
                    }
                } else {
                    if let Ok(mut fs) = mount.filesystem.root() {
                        result.extend(fs.list()?.iter().map(|s| s.clone()))
                    }
                }

                Ok(result)
            })
    }

    fn metadata(&mut self, _: &str) -> io::Result<Box<dyn Metadata2>> {
        ioerr!(Unsupported)
    }
}

//TODO: remove
unsafe impl Sync for VirtualFileSystem {}

unsafe impl Send for VirtualFileSystem {}
