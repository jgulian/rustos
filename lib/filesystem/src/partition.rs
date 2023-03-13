#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
use shim::io;

//TODO: this has a huge design flaw: i.e. it can't
// handle more than one filesystem on the same device

use crate::device::BlockDevice;
use crate::error::FilesystemError;
use crate::master_boot_record::MasterBootRecord;

pub struct BlockPartition {
    /// The device the partition is based on
    device: Box<dyn BlockDevice>,
    /// The physical sector where the partition begins.
    block_offset: u64,
    /// Number of sectors
    block_count: u64,
    /// The size, in bytes, of a logical sector in the partition.
    block_size: u64,
}

impl BlockPartition {
    /// Set the block size of the block partition
    pub fn set_block_size(&mut self, block_size: u64) {
        self.block_size = block_size;
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.block_size / self.device.block_size() as u64
    }

    /// Maps a user's request for a sector `virtual` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virtual_block: u64) -> Option<u64> {
        if virtual_block >= self.block_count {
            return None;
        }

        let physical_offset = virtual_block * self.factor();
        let physical_sector = self.block_offset + physical_offset;

        Some(physical_sector)
    }
}

impl BlockDevice for BlockPartition {
    fn block_size(&self) -> usize {
        self.block_size as usize
    }

    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()> {
        if data.len() != self.block_size as usize {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }

        let physical_block = self.virtual_to_physical(block)
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;

        data.chunks_mut(self.device.block_size())
            .enumerate()
            .try_for_each(|(i, window)| -> io::Result<()> {
                if window.len() == self.device.block_size() {
                    self.device.read_block(physical_block + i as u64, window)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    fn write_block(&mut self, block: u64, data: &[u8]) -> io::Result<()> {
        if data.len() != self.block_size as usize {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }

        let physical_block = self.virtual_to_physical(block)
            .ok_or(io::Error::from(io::ErrorKind::NotFound))?;

        data.chunks(self.device.block_size())
            .enumerate()
            .try_for_each(|(i, window)| -> io::Result<()> {
                if window.len() == self.device.block_size() {
                    self.device.write_block(physical_block + i as u64, window)?;
                }

                Ok(())
            })?;

        Ok(())
    }
}

impl TryFrom<(Box<dyn BlockDevice>, u8)> for BlockPartition {
    type Error = FilesystemError;

    fn try_from((mut device, filesystem): (Box<dyn BlockDevice>, u8)) -> Result<Self, Self::Error> {
        let block_size = device.block_size() as u64;
        let master_boot_record = MasterBootRecord::try_from(&mut device)?;

        let partition = *master_boot_record.partition_table.iter()
            .find(|partition| partition.partition_type == filesystem)
            .ok_or(FilesystemError::BadSignature)?;

        Ok(BlockPartition {
            device,
            block_offset: partition.relative_sector as u64,
            block_count: partition.total_sectors as u64,
            block_size,
        })
    }
}
