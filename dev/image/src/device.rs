use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use filesystem::BlockDevice;

struct ImageFile(File, u16);

impl ImageFile {
    pub fn new(file: File, sector_size: u16) -> ImageFile {
        Self(file, sector_size)
    }

    fn seek_to_sector(&mut self, sector: u64) -> io::Result<u64> {
        self.0.seek(SeekFrom::Start((self.1 as u64 * sector) as u64))
    }

    fn truncate_buffer_length<'a>(&self, buffer: &'a [u8]) -> &'a [u8] {
        if buffer.len() > self.1 as usize {
            &buffer[..self.1 as usize]
        } else {
            buffer
        }
    }

    fn truncate_mut_buffer_length<'a>(&self, buffer: &'a mut [u8]) -> &'a mut [u8] {
        if buffer.len() > self.1 as usize {
            &mut buffer[..self.1 as usize]
        } else {
            buffer
        }
    }
}

impl BlockDevice for ImageFile {
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> shim::io::Result<usize> {
        self.seek_to_sector(n)?;
        let buffer = self.truncate_mut_buffer_length(buf);
        self.0.read_exact(buffer)?;
        Ok(self.1 as usize)
    }

    fn write_sector(&mut self, n: u64, buf: &[u8]) -> shim::io::Result<usize> {
        self.seek_to_sector(n)?;
        let buffer = self.truncate_buffer_length(buf);
        self.0.write_all(buffer)?;
        Ok(self.1 as usize)
    }

    fn flush_sector(&mut self, _: u64) -> shim::io::Result<()> {
        self.0.sync_all()?;
        Ok(())
    }
}