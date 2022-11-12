pub mod sd;

use alloc::rc::Rc;
use alloc::string::String;
use core::fmt::{self, Debug};
use shim::io;
use shim::path::Path;
use filesystem;

use fat32::vfat::{Dir, Entry, File, HandleReference, VFat, VFatHandle};
use crate::multiprocessing::mutex::Mutex;

#[derive(Clone)]
pub struct PiVFatHandle(Rc<Mutex<VFat<Self>>>);

// These impls are *unsound*. We should use `Arc` instead of `Rc` to implement
// `Sync` and `Send` trait for `PiVFatHandle`. However, `Arc` uses atomic memory
// access, which requires MMU to be initialized on ARM architecture. Since we
// have enabled only one core of the board, these unsound impls will not cause
// any immediate harm for now. We will fix this in the future.
unsafe impl Send for PiVFatHandle {}
unsafe impl Sync for PiVFatHandle {}

impl Debug for PiVFatHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "PiVFatHandle")
    }
}

impl VFatHandle for PiVFatHandle {
    fn new(val: VFat<PiVFatHandle>) -> Self {
        PiVFatHandle(Rc::new(Mutex::new(val)))
    }

    fn lock<R>(&self, f: impl FnOnce(&mut VFat<PiVFatHandle>) -> R) -> R {
        f(&mut self.0.lock())
    }
}

pub struct FileSystem(Mutex<Option<PiVFatHandle>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    /// The caller should assure that the method is invoked only once during the
    /// kernel2 initialization.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub unsafe fn initialize(&self) {
        let sd = sd::Sd::new().expect("filesystem failed to initialize");
        let vfat = VFat::<PiVFatHandle>::from(sd).expect("failed to initialize vfat");
        self.0.lock().replace(vfat);
    }

    pub fn create_file(self, name: String) -> io::Result<File<PiVFatHandle>> {
        use filesystem::FileSystem;
        HandleReference(self.0.lock().as_ref().unwrap()).new_file(name)
    }
}

impl filesystem::FileSystem for &FileSystem {
    type File = File<PiVFatHandle>;
    type Dir = Dir<PiVFatHandle>;
    type Entry = Entry<PiVFatHandle>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        HandleReference(self.0.lock().as_ref().unwrap()).open(path)
    }

    fn new_file(&mut self, name: String) -> io::Result<Self::File> {
        HandleReference(self.0.lock().as_ref().unwrap()).new_file(name)
    }

    fn new_dir(&mut self, name: String) -> io::Result<Self::Dir> {
        HandleReference(self.0.lock().as_ref().unwrap()).new_dir(name)
    }
}