use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{ToString};
use alloc::sync::Arc;

use core::cell::UnsafeCell;
use core::fmt::{self, Debug};

use fat32::vfat::{HandleReference, VFat, VFatHandle};
use filesystem;
use filesystem::devices::CharDeviceFileSystem;
use filesystem::fs2::{Directory2, FileSystem2};
use filesystem::path::Path;
use filesystem::{CharDevice, VirtualFileSystem};
use pi::uart::MiniUart;
use shim::{io, newioerr};
use shim::io::{Read, Write};


use crate::FILESYSTEM;
use crate::multiprocessing::mutex::Mutex;

pub mod sd;

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

struct PiVFatWrapper(UnsafeCell<Option<PiVFatHandle>>);

impl PiVFatWrapper {
    pub const fn uninitialized() -> Self {
        PiVFatWrapper(UnsafeCell::new(None))
    }

    pub unsafe fn initialize(&self) {
        let sd = sd::Sd::new().expect("persistent failed to initialize");
        let vfat = VFat::<PiVFatHandle>::from(sd).expect("failed to initialize vfat");
        (&mut *self.0.get()).replace(vfat);
    }

    fn handle(&self) -> &PiVFatHandle {
        unsafe { self.0.get().as_ref() }.unwrap().as_ref().unwrap()
    }
}

unsafe impl Sync for PiVFatWrapper {}

static PI_VFAT_HANDLE_WRAPPER: PiVFatWrapper = PiVFatWrapper::uninitialized();

pub struct DiskFileSystem<'a>(&'a PiVFatHandle);

impl<'a> FileSystem2 for DiskFileSystem<'a> {
    fn root(&mut self) -> io::Result<Box<dyn Directory2>> {
        HandleReference(self.0).root()
    }

    fn copy_entry(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        HandleReference(self.0).copy_entry(source, destination)
    }
}

pub struct FileSystem(Mutex<Option<VirtualFileSystem>>);

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
        self.0.lock().replace(VirtualFileSystem::new());
        PI_VFAT_HANDLE_WRAPPER.initialize();

        let disk_file_system = Box::new(DiskFileSystem(PI_VFAT_HANDLE_WRAPPER.handle()));
        FILESYSTEM.0.lock().as_mut().unwrap().mount(Path::root(), disk_file_system);

        let console_path = Path::root();

        let console_filesystem = Box::new(CharDeviceFileSystem::new(
            "console".to_string(), ConsoleFile::new())
        );
        FILESYSTEM.0.lock().as_mut().unwrap().mount(console_path, console_filesystem);
    }
}

impl FileSystem2 for &FileSystem {
    fn root(&mut self) -> io::Result<Box<dyn Directory2>> {
        self.0.lock().as_mut().ok_or(newioerr!(Unsupported))?.root()
    }

    fn copy_entry(&mut self, source: &Path, destination: &Path) -> io::Result<()> {
        self.0.lock().as_mut().ok_or(newioerr!(Unsupported))?.copy_entry(source, destination)
    }
}

struct ConsoleFile(Arc<Mutex<MiniUart>>);

impl Read for ConsoleFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.lock().read(buf)
    }
}

impl Write for ConsoleFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.lock().flush()
    }
}

impl Clone for ConsoleFile {
    fn clone(&self) -> Self {
        ConsoleFile(self.0.clone())
    }
}

impl CharDevice for ConsoleFile {}

impl ConsoleFile {
    fn new() -> Self {
        ConsoleFile(Arc::new(Mutex::new(MiniUart::new())))
    }
}