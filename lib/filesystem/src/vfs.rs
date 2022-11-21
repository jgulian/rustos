use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use shim::path::{Path, PathBuf};
use shim::{io, ioerr, newioerr};
use crate::fs2;
use crate::fs2::{Directory2, Entry2, FileSystem2, Metadata2};

struct Mount {
    mount_point: PathBuf,
    filesystem: Box<dyn FileSystem2>,
}

//TODO: this is not thread safe
struct Mounts(Rc<Vec::<Mount>>);

pub struct VirtualFileSystem {
    mounts: Mounts,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        VirtualFileSystem {
            mounts: Mount(Rc::new(Vec::<Mount>::new())),
        }
    }

    pub fn mount(&mut self, mount_point: PathBuf, filesystem: Box<dyn FileSystem2>) {
        self.filesystems.push(VFSEntry {
            mount_point,
            filesystem,
        })
    }
}

impl FileSystem2 for VirtualFileSystem {
    fn root(&mut self) -> io::Result<Box<dyn Directory2>> {
        todo!()
    }

    fn copy_entry(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        todo!()
    }
}

struct VFSDirectory {
    path: PathBuf,
    mounts: Mounts,
}

impl Directory2 for VFSDirectory {
    fn open_entry(&mut self, _: &str) -> io::Result<Entry2> {

    }

    fn create_file(&mut self, _: &str) -> io::Result<()> {
        ioerr!(Unsupported)
    }

    fn create_directory(&mut self, _: &str) -> io::Result<()> {
        ioerr!(Unsupported)
    }

    fn remove(&mut self, _: &str) -> io::Result<()> {
        ioerr!(Unsupported)
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        self.mounts.0.iter().map(|mount| {
            mount.mount_point
        })
    }

    fn metadata(&mut self, _: &str) -> io::Result<Box<dyn Metadata2>> {
        ioerr!(Unsupported)
    }
}


// impl FileSystem for VirtualFileSystem {
//     type File = Box<DynFile>;
//     type Dir = Box<DynDir>;
//     type Entry = Box<DynEntry>;
//
//     fn open(&mut self, path: &Path) -> io::Result<Self::Entry> {}
//
//     fn new_file(&mut self, path: &Path) -> io::Result<Self::File> {
//         todo!()
//     }
//
//     fn new_dir(&mut self, path: &Path) -> io::Result<Self::Dir> {
//         todo!()
//     }
// }

//
//trait GenericFilesystem {
//    fn new_file2(&mut self, path: &Path) -> io::Result<Box<dyn File>>;
//}
//
//impl<T: FileSystem> GenericFilesystem for Box<T> {
//    fn new_file2(&mut self, path: &Path) -> io::Result<Box<dyn File>> {
//        self.new_file(path).map(|file| Box::new(file))
//    }
//}
//