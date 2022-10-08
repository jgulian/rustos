use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::min;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub starting_sector: Cluster,
    pub name: String,
    pub metadata: Metadata,
    pub file_size: u64,
}

impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }

    fn size(&self) -> u64 {
        self.file_size
    }
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!("not required")
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }
}


impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut data = Vec::<u8>::new();
        self.vfat.lock(|file_system|
            file_system.read_chain(self.starting_sector, &mut data))?;

        buf.copy_from_slice(data.as_slice());

        Ok(min(buf.len(), data.len()))
    }
}

impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}
