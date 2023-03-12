use shim::io;

use alloc::vec;

pub trait ByteDevice {
    fn read_byte(&mut self) -> io::Result<u8>;
    fn write_byte(&mut self, byte: u8) -> io::Result<()>;
}

pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()>;
    fn write_block(&mut self, block: u64, data: &[u8]) -> io::Result<()>;
}

pub fn stream_read<I>(device: &mut dyn BlockDevice, offset: usize, blocks: I, data: &mut [u8]) -> io::Result<(u64, usize)> where I: Iterator<Item=u64> {
    let block_size = device.block_size();
    let (block_offset, small_offset) = offset_data(offset, block_size);
    let (prefix_data, main_data) = data.split_at_mut(small_offset);
    let (mut last_block, mut amount_read) = (0, 0);

    blocks.skip(block_offset)
        .zip([prefix_data].into_iter().chain(main_data.chunks_mut(block_size)))
        .enumerate()
        .try_for_each(|(i, (block, chunk_data))| {
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
    let (prefix_data, main_data) = data.split_at(small_offset);
    let (mut last_block, mut amount_written) = (0, 0);

    blocks.skip(block_offset)
        .zip([prefix_data].into_iter().chain(main_data.chunks(block_size)))
        .enumerate()
        .try_for_each(|(i, (block, chunk_data))| {
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