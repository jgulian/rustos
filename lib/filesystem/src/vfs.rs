use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;
use shim::path::{Path, PathBuf};
use shim::{io, newioerr};
use crate::{Dir, Entry, File, FileSystem, Metadata, Timestamp};

//struct VFSEntry {
//    prefix: PathBuf,
//    filesystem: Box<dyn GenericFilesystem>,
//}
//
//pub struct VirtualFileSystem {
//    filesystems: Vec::<VFSEntry>,
//}
//
//impl VirtualFileSystem {
//    //pub fn register<T: FileSystem>(&mut self, prefix: PathBuf, filesystem: Box<dyn T>) {
//    //    self.filesystems.push(VFSEntry {
    //        prefix,
    //        filesystem,
    //    })
    //}

    // fn find_filesystem<'a>(&'a mut self, path: &Path) -> io::Result<&'a mut Box<DynFilesystem>> {
    //     self.filesystems.iter_mut().enumerate()
    //         .filter_map(|(i, entry)| -> Option<&'a mut DynFilesystem> {
    //             if path.starts_with(entry.prefix.as_path()) {
    //                 Some(entry)
    //             } else {
    //                 None
    //             }
    //         }).next().ok_or(newioerr!(InvalidFilename))
    // }
//}


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