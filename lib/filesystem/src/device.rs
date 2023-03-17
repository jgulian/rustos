#[cfg(feature = "no_std")]
use alloc::vec;
use log::info;
#[cfg(not(feature = "no_std"))]
use std::vec;

use shim::io;
use shim::io::SeekFrom;

pub trait ByteDevice {
    fn read_byte(&mut self) -> io::Result<u8> {
        loop {
            match self.try_read_byte() {
                Ok(byte) => return Ok(byte),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
        }
    }
    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        loop {
            match self.try_write_byte(byte) {
                Ok(_) => return Ok(()),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(e),
            }
        }
    }

    fn try_read_byte(&mut self) -> io::Result<u8>;
    fn try_write_byte(&mut self, byte: u8) -> io::Result<()>;
}

impl io::Read for dyn ByteDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut count = 0;
        let result = buf.iter_mut().try_for_each(|byte| -> io::Result<()> {
            *byte = self.try_read_byte()?;
            count += 1;
            Ok(())
        });

        match result {
            Ok(_) => return Ok(count),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(count),
            Err(e) => Err(e),
        }
    }
}

impl io::Write for dyn ByteDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut count = 0;
        let result = buf.iter().try_for_each(|byte| -> io::Result<()> {
            self.try_write_byte(*byte)?;
            count += 1;
            Ok(())
        });

        match result {
            Ok(_) => return Ok(count),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(count),
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Seek for dyn ByteDevice {
    fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
        Err(io::Error::from(io::ErrorKind::NotSeekable))
    }
}

pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()>;
    fn write_block(&mut self, block: u64, data: &[u8]) -> io::Result<()>;
}

pub fn stream_read<I>(device: &mut dyn BlockDevice, offset: usize, blocks: I, data: &mut [u8]) -> io::Result<(u64, usize)> where I: Iterator<Item=u64> {
    let block_size = device.block_size();
    let (block_offset, small_offset) = offset_data(offset, block_size);
    let (prefix_data, main_data) = data.split_at_mut(small_offset % block_size);
    let (mut last_block, mut amount_read) = (0, 0);

    blocks.skip(block_offset)
        .zip([prefix_data].into_iter().chain(main_data.chunks_mut(block_size)))
        .enumerate()
        .try_for_each(|(i, (block, chunk_data))| -> io::Result<()> {
            if chunk_data.len() == block_size {
                device.read_block(block, chunk_data)?;
            } else {
                let mut shadow = vec![0u8; block_size];
                device.read_block(block, shadow.as_mut_slice())?;
                let shadow_offset = if i == 0 { small_offset } else { 0 };
                let chunk_shadow = &shadow[shadow_offset..shadow_offset + chunk_data.len()];
                chunk_data.copy_from_slice(chunk_shadow);
            }

            last_block = block;
            amount_read += chunk_data.len();
            Ok(())
        })?;

    Ok((last_block, amount_read))
}

pub fn stream_write<I>(device: &mut dyn BlockDevice, offset: usize, blocks: I, data: &[u8]) -> io::Result<(u64, usize)> where I: Iterator<Item=u64> {
    let block_size = device.block_size();
    let (block_offset, small_offset) = offset_data(offset, block_size);
    let (prefix_data, main_data) = data.split_at(small_offset % block_size);
    let (mut last_block, mut amount_written) = (0, 0);

    blocks.skip(block_offset)
        .zip([prefix_data].into_iter().chain(main_data.chunks(block_size)))
        .enumerate()
        .try_for_each(|(i, (block, chunk_data))| -> io::Result<()> {
            if chunk_data.len() == block_size {
                device.write_block(block, chunk_data)?;
            } else {
                let mut shadow = vec![0u8; block_size];
                device.read_block(block, shadow.as_mut_slice())?;
                let shadow_offset = if i == 0 { small_offset } else { 0 };
                let chunk_shadow = &mut shadow[shadow_offset..shadow_offset + chunk_data.len()];
                chunk_shadow.copy_from_slice(chunk_data);
                device.write_block(block, shadow.as_slice())?;
            }

            last_block = block;
            amount_written += chunk_data.len();
            Ok(())
        })?;

    Ok((last_block, amount_written))
}

fn offset_data(offset: usize, block_size: usize) -> (usize, usize) {
    (offset / block_size, block_size - offset % block_size)
}

fn get_chunk(block_size: usize, offset: usize, buffer: &mut [u8]) -> (usize, &mut [u8]) {
    if offset == block_size {
        (offset + block_size, &mut buffer[..block_size])
    } else {
        (0, &mut buffer[offset..block_size])
    }
}