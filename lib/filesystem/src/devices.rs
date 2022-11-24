use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::ops::DerefMut;

use shim::{io, ioerr};
use shim::io::{Read, Seek, SeekFrom, Write};

use crate::{File, FileSystem};
use crate::fs2::{Directory2, Entry2, File2, FileSystem2, Metadata2};
use crate::path::Path;

/// Trait implemented by devices that can be read/written in sector
/// granularities.
pub trait BlockDevice: Send {
    /// Sector size in bytes. Must be a multiple of 512 >= 512. Defaults to 512.
    fn sector_size(&self) -> u64 {
        512
    }

    /// Read sector number `n` into `buf`.
    ///
    /// `self.sector_size()` or `buf.len()` bytes, whichever is less, are read
    /// into `buf`. The number of bytes read is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or reading from `self` fails.
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize>;

    /// Append sector number `n` into `vec`.
    ///
    /// `self.sector_size()` bytes are appended to `vec`. The number of bytes
    /// read is returned.
    ///
    /// FIXME: This can probably be deleted
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or reading from `self` fails.
    fn read_all_sector(&mut self, n: u64, vec: &mut Vec<u8>) -> io::Result<usize> {
        let sector_size = self.sector_size() as usize;

        let start = vec.len();
        vec.reserve(sector_size);

        unsafe {
            vec.set_len(start + sector_size);
        }
        // XXX. handle: clean-up dirty data when failed
        let read = self.read_sector(n, &mut vec[start..])?;
        unsafe {
            vec.set_len(start + read);
        }
        Ok(read)
    }

    /// Overwrites sector `n` with the contents of `buf`.
    ///
    /// `self.sector_size()` or `buf.len()` bytes, whichever is less, are written
    /// to the sector. The number of byte written is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if seeking or writing to `self` fails. Returns an
    /// error of `UnexpectedEof` if the length of `buf` is less than
    /// `self.sector_size()`.
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize>;

    fn flush_sector(&mut self, n: u64) -> io::Result<()>;
}

impl<'a, T: BlockDevice> BlockDevice for &'a mut T {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
        (*self).read_sector(n, buf)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
        (*self).write_sector(n, buf)
    }

    fn flush_sector(&mut self, n: u64) -> io::Result<()> {
        (*self).flush_sector(n)
    }
}


//FIXME: this can probably be deleted
macro impl_for_read_write_seek($(<$($gen:tt),*>)* $T:path) {
impl $(<$($gen),*>)* BlockDevice for $T {
        fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize> {
            use shim::io::{Read, Seek, SeekFrom};
            let sector_size = self.sector_size();
            let to_read = ::core::cmp::min(sector_size as usize, buf.len());
            self.seek(SeekFrom::Start(n * sector_size))?;
            self.read_exact(&mut buf[..to_read])?;
            Ok(to_read)
        }

        fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize> {
            use shim::io::{Write, Seek, SeekFrom};
            let sector_size = self.sector_size();
            let to_write = ::core::cmp::min(sector_size as usize, buf.len());
            self.seek(SeekFrom::Start(n * sector_size))?;
            self.write_all(&buf[..to_write])?;
            Ok(to_write)
        }

        fn flush_sector(&mut self, n: u64) -> io::Result<()> {
            Ok(())
        }
    }
}

impl_for_read_write_seek!(<'a> shim::io::Cursor<&'a mut [u8]>);
impl_for_read_write_seek!(shim::io::Cursor<Vec<u8>>);
impl_for_read_write_seek!(shim::io::Cursor<Box<[u8]>>);
#[cfg(test)]
impl_for_read_write_seek!(::std::fs::File);

pub trait CharDevice: Send + Read + Write {}

pub struct CharDeviceFileSystem<T: CharDevice + 'static>(String, Arc<RefCell<(T, bool)>>);

pub struct CharDeviceRootDirectory<T: CharDevice + 'static>(String, Arc<RefCell<(T, bool)>>);

pub struct CharDeviceFile<T: CharDevice + 'static>(Arc<RefCell<(T, bool)>>);

impl<T: CharDevice + 'static> CharDeviceFileSystem<T> {
    pub fn new(name: String, device: T) -> Self {
        CharDeviceFileSystem {
            0: name,
            1: Arc::new(RefCell::new((device, false))),
        }
    }
}

impl<T: CharDevice + 'static> FileSystem2 for CharDeviceFileSystem<T> {
    fn root(&mut self) -> io::Result<Box<dyn Directory2>> {
        Ok(Box::new(CharDeviceRootDirectory(self.0.clone(), self.1.clone())))
    }

    fn copy_entry(&mut self, _: &Path, _: &Path) -> io::Result<()> {
        ioerr!(Unsupported)
    }
}

impl<T: CharDevice + 'static> Directory2 for CharDeviceRootDirectory<T> {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry2> {
        if self.0.eq(name) {
            let mut binding = self.1.as_ref().borrow_mut();
            let (_, busy) = binding.deref_mut();
            if !*busy {
                *busy = true;
                Ok(Entry2::File(Box::new(CharDeviceFile(self.1.clone()))))
            } else {
                ioerr!(ResourceBusy)
            }
        } else {
            ioerr!(NotFound)
        }
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
        Ok(vec![self.0.clone()])
    }

    fn metadata(&mut self, _: &str) -> io::Result<Box<dyn Metadata2>> {
        ioerr!(Unsupported)
    }
}

impl<T: CharDevice + 'static> Seek for CharDeviceFile<T> {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        ioerr!(NotSeekable)
    }
}

impl<T: CharDevice + 'static> Read for CharDeviceFile<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut binding = self.0.as_ref().borrow_mut();
        let (device, _) = binding.deref_mut();
        device.read(buf)
    }
}

impl<T: CharDevice + 'static> Write for CharDeviceFile<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut binding = self.0.as_ref().borrow_mut();
        let (device, _) = binding.deref_mut();
        device.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: CharDevice + 'static> File2 for CharDeviceFile<T> {}

impl<T: CharDevice + 'static> Drop for CharDeviceFile<T> {
    fn drop(&mut self) {
        let mut binding = self.0.as_ref().borrow_mut();
        let (_, busy) = binding.deref_mut();
        *busy = true;
    }
}