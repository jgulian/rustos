#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::vec;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::sync::Arc;
#[cfg(not(feature = "no_std"))]
use std::vec;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use core::mem;
use log::info;

use filesystem::device::{BlockDevice, stream_read, stream_write};
use filesystem::error::FilesystemError;
use filesystem::partition::BlockPartition;
use shim::io;
use sync::Mutex;
use crate::cluster::Cluster;
use crate::bios_parameter_block::BiosParameterBlock;
use crate::chain::Chain;
use crate::directory::Directory;
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
    fn get_blocks(&self, cluster: Cluster) -> (u64, u64) {
        let mut first_block = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let final_block = first_block + self.sectors_per_cluster as u64;
        (first_block, final_block)
    }

    pub(crate) fn read_cluster(&mut self, cluster: Cluster, offset: usize, buffer: &mut [u8]) -> io::Result<usize> {
        let (first_block, final_block) = self.get_blocks(cluster);
        stream_read(&mut self.device, offset, first_block..final_block, buffer)
            .map(|(_, amount)| amount)
    }

    pub(crate) fn write_cluster(&mut self, cluster: Cluster, offset: usize, buffer: &[u8]) -> io::Result<usize> {
        let (first_block, final_block) = self.get_blocks(cluster);
        stream_write(&mut self.device, offset, first_block..final_block, buffer)
            .map(|(_, amount)| amount)
    }

    pub(crate) fn fat_entry(&mut self, cluster: Cluster) -> VirtualFatResult<FatEntry> {
        let (block, offset) = FatEntry::find(cluster, self.fat_start_sector, self.bytes_per_sector as u64);

        let mut data = vec![0u8; self.device.block_size() as usize];
        self.device.read_block(block, data.as_mut_slice())?;

        let mut fat_data = [0u8; 4];
        fat_data.copy_from_slice(&data[offset..offset + 4]);

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

    pub(crate) fn fat_chain(&mut self, mut cluster: Cluster) -> VirtualFatResult<Vec<Cluster>> {
        let mut result = Vec::new();

        loop {
            result.push(cluster);
            match self.fat_entry(cluster)?.status() {
                Status::Data(next_cluster) => {
                    cluster = next_cluster;
                }
                Status::Eoc(_) => {
                    return Ok(result);
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

pub struct VirtualFatFilesystem<M: Mutex<VirtualFat>>(Arc<M>);

impl<M: Mutex<VirtualFat> + 'static> filesystem::filesystem::Filesystem for VirtualFatFilesystem<M> {
    fn root(&mut self) -> io::Result<Box<dyn filesystem::filesystem::Directory>> {
        let virtual_fat = self.0.clone();

        let root_cluster = self.0.lock(|virtual_fat| virtual_fat.root_cluster)
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

        let chain = Chain::new_from_cluster(virtual_fat.clone(), root_cluster)
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        Ok(Box::new(Directory {
            virtual_fat,
            metadata: Default::default(),
            chain,
        }))
    }

    fn format(device: &mut dyn BlockDevice, partition: &mut filesystem::master_boot_record::PartitionEntry, sector_size: usize) -> io::Result<()> where Self: Sized {
        todo!()
    }
}

impl<M: Mutex<VirtualFat> + 'static> VirtualFatFilesystem<M> {
    pub fn new(mut value: BlockPartition) -> Result<Self, FilesystemError> {
        let bios_parameter_block = BiosParameterBlock::try_from(&mut value)?;
        value.set_block_size(bios_parameter_block.bytes_per_sector as u64);

        let sectors_per_fat = if bios_parameter_block.sectors_per_fat_one != 0 {
            bios_parameter_block.sectors_per_fat_one as u32
        } else {
            bios_parameter_block.sectors_per_fat_two
        };

        let data_start_sector = bios_parameter_block.reserved_sectors as u64 +
            (bios_parameter_block.number_of_fats as u64 * sectors_per_fat as u64);

        Ok(VirtualFatFilesystem(Arc::new(M::new(VirtualFat {
            device: value,
            bytes_per_sector: bios_parameter_block.bytes_per_sector,
            sectors_per_cluster: bios_parameter_block.sectors_per_cluster,
            sectors_per_fat,
            fat_start_sector: bios_parameter_block.reserved_sectors as u64,
            data_start_sector: data_start_sector as u64,
            root_cluster: Cluster::from(bios_parameter_block.root_cluster),
        }))))
    }
}