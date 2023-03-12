use alloc::vec;
use core::mem;

use filesystem::device::{BlockDevice, stream_read, stream_write};
use filesystem::error::FilesystemError;
use filesystem::partition::BlockPartition;
use shim::io;
use crate::cluster::Cluster;
use crate::bios_parameter_block::BiosParameterBlock;
use crate::error::{VirtualFatError, VirtualFatResult};
use crate::fat::{FatEntry, Status};

pub struct VirtualFat {
    device: BlockPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_cluster: Cluster,
}

impl VirtualFat {
    fn get_blocks(&self, cluster: Cluster, offset: usize) -> (u64, u64) {
        let mut first_block = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let final_block = first_block + self.sectors_per_cluster as u64;
        (first_block, final_block)
    }

    pub(crate) fn read_cluster(&mut self, cluster: Cluster, offset: usize, buffer: &mut [u8]) -> io::Result<usize> {
        let (first_block, final_block) = self.get_blocks(cluster, offset);
        stream_read(&mut self.device, offset, first_block..final_block, buffer)
    }

    pub(crate) fn write_cluster(&mut self, cluster: Cluster, offset: usize, buffer: &[u8]) -> io::Result<usize> {
        let (first_block, final_block) = self.get_blocks(cluster, offset);
        stream_write(&mut self.device, offset, first_block..final_block, buffer)
    }

    pub fn fat_entry(&mut self, cluster: Cluster) -> VirtualFatResult<FatEntry> {
        let (block, offset) = FatEntry::find(cluster, self.fat_start_sector, self.bytes_per_sector as u64);

        let mut data = vec![0u8; self.device.block_size() as usize];
        self.device.read_block(block, data.as_mut_slice())?;

        let fat_data = [0u8; 4];
        fat_data.copy_from_slice(&data[offset..]);

        Ok(FatEntry::from(fat_data))
    }

    pub(crate) fn next_cluster(&mut self, cluster: Cluster) -> VirtualFatResult<Option<Cluster>> {
        match self.fat_entry(cluster)?.status() {
            Status::Eoc(_status) => {
                Ok(None)
            }
            Status::Data(next_cluster) => {
                Ok(Some(next_cluster))
            }
            _ => {
                Err(VirtualFatError::InvalidClusterForNext)
            }
        }
    }

    pub(crate) fn next_free_cluster(&mut self) -> VirtualFatResult<Cluster> {
        let number_of_fats = self.number_of_fats() as u32;
        (0..number_of_fats).find_map(|fat| -> Option<VirtualFatResult<Cluster>> {
            match self.fat_entry(Cluster::from(fat)) {
                Ok(fat_entry) => {
                    if fat_entry.is_free() {
                        Some(Ok(Cluster::from(fat)))
                    } else {
                        None
                    }
                }
                Err(err) => Some(Err(err))
            }
        }).ok_or(VirtualFatError::FilesystemOutOfMemory)?
    }

    pub(crate) fn update_fat_entry(&mut self, cluster: Cluster, status: Status) -> VirtualFatResult<()> {
        let (block, offset) = FatEntry::find(cluster, self.fat_start_sector, self.bytes_per_sector as u64);

        let mut data = vec![0u8; self.device.block_size() as usize];
        self.device.read_block(block, data.as_mut_slice())?;

        let fat_entry: [u8; 4] = FatEntry::from(status).into();
        (&mut data[offset..offset + 4]).copy_from_slice(&fat_entry);

        self.device.write_block(block, data.as_slice())?;

        Ok(())
    }
    
    pub(crate) fn fat_chain_length(&mut self, mut cluster: Cluster) -> VirtualFatResult<u64> {
        let mut length = 0;
        
        loop {
            length += self.block_size() as u64;
            match self.fat_entry(cluster)?.status() {
                Status::Data(next_cluster) => {
                    cluster = next_cluster;
                }
                Status::Eoc(_) => {
                    return Ok(length);
                }
                _ => return Err(VirtualFatError::InvalidFatForSizing),
            }
        }
    }

    fn number_of_fats(&self) -> usize {
        self.sectors_per_fat as usize * self.bytes_per_sector as usize / mem::size_of::<FatEntry>()
    }
}

impl BlockDevice for VirtualFat {
    fn block_size(&self) -> usize {
        self.bytes_per_sector as usize * self.sectors_per_cluster as usize
    }

    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()> {
        if data.len() != self.block_size() {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }

        self.read_cluster(Cluster::from(block as u32), 0, data)?;
        Ok(())
    }

    fn write_block(&mut self, block: u64, data: &[u8]) -> io::Result<()> {
        if data.len() != self.block_size() {
            return Err(io::Error::from(io::ErrorKind::Unsupported));
        }

        self.write_cluster(Cluster::from(block as u32), 0, data)?;
        Ok(())
    }
}

impl TryFrom<BlockPartition> for VirtualFat {
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

        Ok(VirtualFat {
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