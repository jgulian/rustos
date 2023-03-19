use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::sync::Arc;

use core::cell::UnsafeCell;
use core::fmt::{self, Debug};

use filesystem;
use filesystem::cache::CachedBlockDevice;
use filesystem::filesystem::{Filesystem, Directory, File};
use filesystem::device::{BlockDevice, ByteDevice};
use filesystem::master_boot_record::{MasterBootRecord, PartitionEntry};
use filesystem::partition::BlockPartition;
use filesystem::path::Path;
use filesystem::virtual_file_system::{ByteDeviceFilesystem, Mounts, VirtualFilesystem};
use vfat::virtual_fat::{VirtualFat, VirtualFatFilesystem};

use pi::uart::MiniUart;
use shim::{io, ioerr, newioerr};
use shim::io::{Read, Write};
use sync::Mutex;
use crate::disk::sd::Sd;

use crate::FILESYSTEM;
use crate::multiprocessing::spin_lock::SpinLock;

pub mod sd;

pub struct FileSystem(SpinLock<Option<VirtualFilesystem<SpinLock<Mounts>>>>);

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
        let mut virtual_file_system = VirtualFilesystem::default();

        let mut sd_device = Sd::new().unwrap();
        let mut cached_sd_device = CachedBlockDevice::new(sd_device, None);
        let virtual_fat_block_partition = BlockPartition::new(Box::new(cached_sd_device), 0xc).unwrap();
        let disk_file_system = VirtualFatFilesystem::<SpinLock<VirtualFat>>::new(virtual_fat_block_partition).unwrap();

        virtual_file_system.mount(Path::root(), Box::new(disk_file_system)).unwrap();

        let console_path = Path::root();
        let console_filesystem = Box::new(
            ByteDeviceFilesystem::new(ConsoleFile::new(), String::from("console"))
        );
        virtual_file_system.mount(console_path, console_filesystem).unwrap();

        self.0.lock(|filesystem|{
            filesystem.replace(virtual_file_system);
        }).unwrap();
    }
}

impl Filesystem for &FileSystem {
    fn root(&mut self) -> io::Result<Box<dyn Directory>> {
        self.0.lock(|filesystem| {
            match filesystem {
                None => ioerr!(Unsupported),
                Some(fs) => fs.root()
            }
        }).unwrap()
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
        info!("reading byte");
        Ok(self.0.lock(|byte_device| byte_device.read_byte()).unwrap())
    }

    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        Ok(self.0.lock(|byte_device| byte_device.write_byte(byte)).unwrap())
    }

    fn try_read_byte(&mut self) -> io::Result<u8> {
        self.0.lock(|byte_device| {
            if byte_device.has_byte() {
                Ok(byte_device.read_byte())
            } else {
                Err(io::Error::from(io::ErrorKind::WouldBlock))
            }
        }).unwrap()
    }

    fn try_write_byte(&mut self, byte: u8) -> io::Result<()> {
        self.0.lock(|byte_device| {
            if byte_device.can_write() {
                Ok(byte_device.write_byte(byte))
            } else {
                Err(io::Error::from(io::ErrorKind::WouldBlock))
            }
        }).unwrap()
    }
}

impl Clone for ConsoleFile {
    fn clone(&self) -> Self {
        ConsoleFile(self.0.clone())
    }
}