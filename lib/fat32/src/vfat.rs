use alloc::vec;
use core::mem;

use std::ops::Deref;
use filesystem::device::BlockDevice;
use filesystem::error::FilesystemError;
use filesystem::partition::BlockPartition;
use shim::io;
use crate::cluster::Cluster;
use crate::ebpb::BiosParameterBlock;
use crate::fat::{FatEntry, Status};

pub struct VFat {
    device: BlockPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_cluster: Cluster,
}

impl VFat {
    fn get_blocks(&self, cluster: Cluster, offset: usize) -> (u64, u64) {
        let mut first_block = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let final_block = first_block + self.sectors_per_cluster as u64;
        first_block += offset as u64 / self.bytes_per_sector as u64;
        (first_block, final_block)
    }

    fn get_offset(&self, offset: usize) -> (usize, usize) {
        let bytes_per_sector = self.bytes_per_sector as usize;
        (bytes_per_sector, bytes_per_sector - offset % bytes_per_sector)
    }

    pub(crate) fn read_cluster(&mut self, cluster: Cluster, offset: usize, buffer: &mut [u8]) -> io::Result<usize> {
        let mut amount_read = 0;

        let (mut first_block, final_block) = self.get_blocks(cluster, offset);
        let (bytes_per_sector, offset_end) = self.get_offset(offset);
        let (offset_data, non_offset) = buffer.split_at_mut(offset_end);

        if !offsetted.is_empty() {
            let mut block = vec![0u8; bytes_per_sector];
            self.device.read_block(first_block, block.as_mut_slice())?;
            offset_data.copy_from_slice(&block.as_slice()[offset..]);

            amount_read += offset_data.len();
            first_block += 1;
        }

        amount_read += non_offset.chunks_mut(bytes_per_sector)
            .zip(first_block..final_block)
            .try_fold(0, |read, (buffer, block_id)| {
                if buffer.len() == bytes_per_sector {
                    self.device.read_block(block_id, buffer)?;
                } else {
                    let mut block = vec![0u8; bytes_per_sector];
                    self.device.read_block(block_id, block.as_mut_slice())?;
                    buffer.copy_from_slice(block.as_slice());
                }
                Ok(read + buffer.len())
            })?;

        Ok(amount_read)
    }

    pub(crate) fn write_cluster(&mut self, cluster: Cluster, offset: usize, buffer: &mut [u8]) -> io::Result<usize> {
        let mut amount_written = 0;

        let (mut first_block, final_block) = self.get_blocks(cluster, offset);
        let (bytes_per_sector, offset_end) = self.get_offset(offset);
        let (offset_data, non_offset) = buffer.split_at(offset_end);

        if !offset_data.is_empty() {
            let mut block = vec![0u8; bytes_per_sector];
            self.device.read_block(first_block, block.as_mut_slice())?;
            &block.as_mut_slice()[offset..].copy_from_slice(offset_data);
            self.device.write_block(first_block, block.as_slice())?;

            amount_written += offset_data.len();
            first_block += 1;
        }

        amount_written += non_offset.chunks(bytes_per_sector)
            .zip(first_block..final_block)
            .try_fold(0, |written, (buffer, block_id)| {
                if buffer.len() == bytes_per_sector {
                    self.device.write_block(block_id, buffer)?;
                } else {
                    let mut block = vec![0u8; bytes_per_sector];
                    self.device.read_block(block_id, block.as_mut_slice())?;
                    block[..buffer.len()].copy_from_slice(buffer);
                    self.device.write_block(block_id, block.as_slice())?;
                }
                Ok(written + buffer.len())
            })?;

        Ok(amount_written)
    }

    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<FatEntry> {
        let sector = self.fat_start_sector + (cluster.offset() / self.bytes_per_sector as u32) as u64;
        let mut data = vec![0u8; self.device.sector_size() as usize];
        self.device.read_sector(sector, data.as_mut_slice())?;
        let offset = (cluster.offset() % self.bytes_per_sector as u32) as usize;
        let entry = &data[offset];
        Ok(*unsafe { mem::transmute::<&u8, &FatEntry>(entry) })
    }

    pub(crate) fn next_cluster(&mut self, cluster: Cluster) -> io::Result<Option<Cluster>> {
        match self.fat_entry(cluster)?.status() {
            Status::Eoc(_status) => {
                Ok(None)
            }
            Status::Data(next_cluster) => {
                Ok(Some(next_cluster))
            }
            _ => {
                
            }
        }
    }
    
    pub(crate) fn next_free_cluster(&mut self) -> io::Result<Cluster> {
        let total_fat_entries = self.sectors_per_fat * (self.device.block_size() / mem::size_of::<FatEntry>()) as u32;
        while !self.fat_entry(Cluster::from(self.empty_fat_pointer))?.is_free() {
            self.empty_fat_pointer = (self.empty_fat_pointer + 1) % total_fat_entries;
        }

        let pointer = self.empty_fat_pointer;
        self.empty_fat_pointer += 1;

        Ok(Cluster::from(pointer))
    }

    pub(crate) fn update_fat_entry(&mut self, cluster: Cluster, status: Status) -> io::Result<()> {
        let sector = self.fat_start_sector + (cluster.offset() / self.bytes_per_sector as u32) as u64;
        let mut data = vec![0u8; self.device.sector_size() as usize];
        self.device.read_sector(sector, data.as_mut_slice())?;

        let offset = (cluster.offset() % self.bytes_per_sector as u32) as usize;
        let entry = &mut data[offset];
        let fat_entry = unsafe { mem::transmute::<&mut u8, &mut FatEntry>(entry) };
        *fat_entry = FatEntry::from(status);

        self.device.write_sector(sector, data.as_mut_slice())?;
        Ok(())
    }
}

impl TryFrom<BlockPartition> for VFat {
    type Error = FilesystemError;

    fn try_from(mut value: BlockPartition) -> Result<Self, Self::Error> {
        let bios_parameter_block = BiosParameterBlock::try_from(&mut value)?;
        value.set_block_size(bios_parameter_block.bytes_per_sector as u64);

        let sectors_per_fat = if bios_parameter_block.sectors_per_fat_one != 0 {
            bios_parameter_block.sectors_per_fat_one as u32
        } else {
            bios_parameter_block.sectors_per_fat_two
        };

        let data_start_sector = bios_parameter_block.reserved_sectors as u64 +
            (bios_parameter_block.number_of_fats as u64 * sectors_per_fat as u64);

        Ok(VFat {
            device: value,
            bytes_per_sector: bios_parameter_block.bytes_per_sector,
            sectors_per_cluster: bios_parameter_block.sectors_per_cluster,
            sectors_per_fat,
            fat_start_sector: bios_parameter_block.reserved_sectors as u64,
            data_start_sector: data_start_sector as u64,
            root_cluster: Cluster::from(bios_parameter_block.root_cluster),
        })
    }
}