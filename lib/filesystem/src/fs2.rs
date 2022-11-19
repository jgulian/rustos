use alloc::boxed::Box;
use alloc::vec::Vec;
use shim::io;
use shim::path::{Path, PathBuf};

trait Metadata2 {

}

// For char devices, their seek just gives a NotSeekable error
trait File2: io::Seek + io::Read + io::Write {}

trait Directory2 {
    fn open_file(&mut self, path: &Path) -> Box<dyn File2>;
    fn open_directory(&mut self, path: &Path) -> Box<dyn Directory2>;

    // create_file/dir
    // remove_file/dir
    // metadata on a path
    // exists on a path
    // copy on a path to another path
    // move on a path to another path
    // list on a path
}

trait FileSystem2 {
    fn root(&mut self) -> Box<dyn Directory2>;
}

struct VFSMount {
    path: PathBuf,
    system: Box<dyn FileSystem2>,
}

struct VFS {
    mounts: Vec<VFSMount>,
}