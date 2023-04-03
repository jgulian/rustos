use crate::device::BlockDevice;
use shim::io as shim_io;
use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct ImageFile(File, usize);

impl ImageFile {
    pub fn new(file: File, block_size: usize) -> ImageFile {
        Self(file, block_size)
    }

    fn seek_to_sector(&mut self, block: u64) -> io::Result<u64> {
        self.0.seek(SeekFrom::Start(self.1 as u64 * block))
    }
}

impl BlockDevice for ImageFile {
    fn block_size(&self) -> usize {
        self.1
    }

    fn read_block(&mut self, block: u64, data: &mut [u8]) -> shim_io::Result<()> {
        if data.len() != self.1 {
            return Err(shim_io::Error::from(shim_io::ErrorKind::Unsupported));
        }

        self.seek_to_sector(block)?;
        self.0.read_exact(data)?;
        Ok(())
    }

    fn write_block(&mut self, block: u64, data: &[u8]) -> shim_io::Result<()> {
        if data.len() != self.1 {
            return Err(shim_io::Error::from(shim_io::ErrorKind::Unsupported));
        }

        self.seek_to_sector(block)?;
        self.0.write_all(data)?;
        Ok(())
    }
}
