use alloc::string::String;

use shim::io::{self, SeekFrom};

use filesystem;
use crate::vfat::{Metadata, VFatHandle};
use crate::vfat::vfat::ChainOffset;

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    pub name: String,
    pub metadata: Metadata,
    pub file_size: u32,
    pub(crate) offset: ChainOffset,
}

impl<HANDLE: VFatHandle> filesystem::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }

    fn size(&self) -> u64 {
        self.file_size as u64
    }
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!("not required")
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }
}


impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset.exhausted {
            return Ok(0);
        }

        let (buffer_to_read, exhausted) = if self.offset.total_bytes + buf.len() > self.file_size as usize {
            (&mut buf[0..(self.file_size as usize - self.offset.total_bytes)], true)
        } else {
            (buf, false)
        };


        let (amount_read, new_offset) = self.vfat.lock(|file_system| {
            file_system.read_chain_offset(buffer_to_read, self.offset.clone())
        })?;

        self.offset = new_offset;
        self.offset.exhausted |= exhausted;
        Ok(amount_read)
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
