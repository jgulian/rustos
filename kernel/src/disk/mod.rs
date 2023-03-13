use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{ToString};
use alloc::sync::Arc;

use core::cell::UnsafeCell;
use core::fmt::{self, Debug};

use filesystem;

use pi::uart::MiniUart;
use shim::{io, ioerr, newioerr};
use shim::io::{Read, Write};

use crate::FILESYSTEM;
use crate::multiprocessing::mutex::Mutex;

pub mod sd;

pub struct DiskFileSystem<'a>(&'a PiVFatHandle);

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

    fn format(_device: &mut dyn BlockDevice, _partition: &mut PartitionEntry, _sector_size: usize) -> io::Result<()> {
        ioerr!(Unsupported)
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