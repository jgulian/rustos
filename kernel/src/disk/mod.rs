use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use core::cell::UnsafeCell;
use core::fmt::{self, Debug};

use filesystem;
use filesystem::filesystem::{Filesystem, Directory, File};
use filesystem::device::{BlockDevice, ByteDevice};
use filesystem::master_boot_record::{MasterBootRecord, PartitionEntry};
use filesystem::partition::BlockPartition;
use filesystem::path::Path;
use filesystem::virtual_file_system::{ByteDeviceFilesystem, VirtualFilesystem};
use vfat::virtual_fat::{VirtualFat, VirtualFatFilesystem};

use pi::uart::MiniUart;
use shim::{io, ioerr, newioerr};
use shim::io::{Read, Write};
use crate::disk::sd::Sd;

use crate::FILESYSTEM;
use crate::multiprocessing::spin_lock::SpinLock;

pub mod sd;

pub struct FileSystem(SpinLock<Option<VirtualFilesystem>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(SpinLock::new(None))
    }

    /// Initializes the file system.
    /// The caller should assure that the method is invoked only once during the
    /// kernel initialization.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub unsafe fn initialize(&self) {
        self.0.lock().replace(VirtualFilesystem::new());

        let mut sd_device = Box::new(Sd::new().unwrap());
        let virtual_fat_block_partition = BlockPartition::try_from((sd_device, 0xc)).unwrap();
        let disk_file_system = VirtualFatFilesystem::<SpinLock<VirtualFat>>::new(virtual_fat_block_partition).unwrap();

        FILESYSTEM.0.lock().as_mut().unwrap().mount(Path::default(), Box::new(disk_file_system));

        let console_path = Path::default();

        let console_filesystem = Box::new(
            ByteDeviceFilesystem::new(ConsoleFile::new(), String::from("console"))
        );
        FILESYSTEM.0.lock().as_mut().unwrap().mount(console_path, console_filesystem);
    }
}

impl Filesystem for &FileSystem {
    fn root(&mut self) -> io::Result<Box<dyn Directory>> {
        self.0.lock()
            .ok_or(newioerr!(Unsupported))
            .and_then(|mut fs| fs.root())
    }

    fn format(_: &mut dyn BlockDevice, _: &mut PartitionEntry, _: usize) -> io::Result<()> where Self: Sized {
        todo!()
    }
}

struct ConsoleFile(Arc<SpinLock<MiniUart>>);

impl ConsoleFile {
    fn new() -> Self {
        ConsoleFile(Arc::new(SpinLock::new(MiniUart::new())))
    }
}

impl ByteDevice for ConsoleFile {
    fn read_byte(&mut self) -> io::Result<u8> {
        Ok(self.0.lock().read_byte())
    }

    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        Ok(self.0.lock().write_byte(byte))
    }

    fn try_read_byte(&mut self) -> io::Result<u8> {
        let mut guard = self.0.lock();
        if guard.has_byte() {
            Ok(guard.read_byte()?)
        } else {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }
    }

    fn try_write_byte(&mut self, byte: u8) -> io::Result<()> {
        let mut guard = self.0.lock();
        if guard.can_write() {
            Ok(guard.write_byte(byte)?)
        } else {
            Err(io::Error::from(io::ErrorKind::WouldBlock))
        }
    }
}