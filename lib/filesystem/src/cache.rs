#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::vec;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use crate::device::BlockDevice;
use shim::io;

struct CacheEntry {
    block: u64,
    data: Vec<u8>,
}

pub struct CachedBlockDevice<T: BlockDevice> {
    device: T,
    cache: Vec<CacheEntry>,
    max_size: Option<usize>,
}

impl<T: BlockDevice> CachedBlockDevice<T> {
    pub fn new(device: T, mut max_size: Option<usize>) -> Self {
        match max_size {
            None => {}
            Some(size) => {
                max_size = if size == 0 { None } else { Some(size) };
            }
        }

        Self {
            device,
            cache: Vec::new(),
            max_size,
        }
    }

    fn get_cache_entry_or_load(&mut self, block: u64) -> io::Result<&mut CacheEntry> {
        let i = self.cache.iter().enumerate().find_map(|(i, cache_entry)| {
            if cache_entry.block == block {
                Some(i)
            } else {
                None
            }
        });
        match i {
            None => {
                self.make_room_in_cache();
                let mut cache_entry = CacheEntry {
                    block,
                    data: vec![0u8; self.block_size()],
                };
                self.device
                    .read_block(cache_entry.block, cache_entry.data.as_mut_slice())?;
                self.cache.push(cache_entry);
                Ok(self.cache.last_mut().unwrap())
            }
            Some(cache_entry) => Ok(&mut self.cache[cache_entry]),
        }
    }

    fn make_room_in_cache(&mut self) {
        match self.max_size {
            None => {}
            Some(size) => {
                if self.cache.len() == size {
                    //TODO: this is essentially lifo, maybe make fifo?
                    self.cache.pop();
                }
            }
        }
    }
}

impl<T: BlockDevice> BlockDevice for CachedBlockDevice<T> {
    fn block_size(&self) -> usize {
        self.device.block_size()
    }

    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()> {
        if data.len() != self.block_size() {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }

        let cache_entry = self.get_cache_entry_or_load(block)?;
        data.copy_from_slice(cache_entry.data.as_slice());
        Ok(())
    }

    fn write_block(&mut self, block: u64, data: &[u8]) -> io::Result<()> {
        if data.len() != self.block_size() {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }

        let cache_entry = self.get_cache_entry_or_load(block)?;
        cache_entry.data.as_mut_slice().copy_from_slice(data);
        Ok(())
    }
}
