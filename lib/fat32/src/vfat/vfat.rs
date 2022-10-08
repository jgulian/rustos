use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;
use core::cmp::min;
use core::mem;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::{Component, Path};

use crate::mbr::MasterBootRecord;
use crate::{mbr, PartitionEntry, traits, vfat};
use crate::traits::{BlockDevice, FileSystem};
use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};

/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
        where
            T: BlockDevice + 'static,
    {
        let master_boot_record =
            MasterBootRecord::from(&mut device)?;
        let mut i = 0;
        for partition in master_boot_record.partition_table.iter() {
            if partition.partition_type == 0xB || partition.partition_type == 0xC {
                break;
            }
            i += 1;
        }
        if i == 4 {
            return Err(Error::NotFound);
        }

        let partition_entry: &PartitionEntry = &master_boot_record.partition_table[i];
        let bios_parameter_block = BiosParameterBlock::from(&mut device, partition_entry.relative_sector as u64)?;

        let partition = Partition {
            start: partition_entry.relative_sector as u64,
            num_sectors: partition_entry.total_sectors as u64,
            sector_size: bios_parameter_block.total_logical_sectors as u64,
        };

        let sectors_per_fat = if bios_parameter_block.sectors_per_fat_one != 0 {
            bios_parameter_block.sectors_per_fat_one as u32
        } else {
            bios_parameter_block.sectors_per_fat_two
        };

        let data_start_sector = bios_parameter_block.reserved_sectors as u64 + (bios_parameter_block.number_of_fats as u64 * sectors_per_fat as u64);

        Ok(HANDLE::new(VFat {
            phantom: Default::default(),
            device: CachedPartition::new(device, partition),
            bytes_per_sector: bios_parameter_block.bytes_per_sector,
            sectors_per_cluster: bios_parameter_block.sectors_per_cluster,
            sectors_per_fat: sectors_per_fat,
            fat_start_sector: bios_parameter_block.reserved_sectors as u64,
            data_start_sector: data_start_sector as u64,
            rootdir_cluster: Cluster::from(bios_parameter_block.root_cluster),
        }))
    }

    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let initial_size = buf.len();
        let mut cluster = start;

        loop {
            let read_into = buf.len();
            buf.resize(buf.len() + self.bytes_per_sector as usize, 0);
            self.read_cluster(cluster, 0, &mut buf.as_mut_slice()[read_into..]);

            match self.fat_entry(cluster)?.status() {
                Status::Eoc(status) => {
                    break;
                },
                Status::Data(next_cluster) => {
                    cluster = next_cluster;
                }
                _ => {
                    panic!("invalid cluster");
                },
            }
        }

        Ok(buf.len() - initial_size)
    }

    pub fn read_cluster(&mut self, cluster: Cluster, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: usize = 0;
        let mut sector_id = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let last_sector = sector_id + self.sectors_per_cluster as u64;

        sector_id += offset as u64 / self.fat_start_sector;
        let mut sector_offset = offset % self.bytes_per_sector as usize;

        while amount_read < buf.len() && sector_id < last_sector {
            let buffer = self.device.get(sector_id)?;
            for i in 0..min(self.bytes_per_sector as usize - sector_offset, buf.len() - amount_read) {
                buf[amount_read + i] = buffer[sector_offset + i];
            }

            sector_offset = 0;
            sector_id += 1;
            amount_read += self.bytes_per_sector as usize - sector_offset;
        }

        Ok(amount_read)
    }

    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let sector_number = cluster.fat_sector_number(self.fat_start_sector, self.bytes_per_sector);
        let entry_offset = cluster.fat_entry_offset(self.bytes_per_sector) as usize;
        let sector = self.device.get(sector_number)?;
        Ok(unsafe {mem::transmute::<&u8, &FatEntry>(&sector[entry_offset])})
    }

    pub fn root_cluster(&self) -> Cluster {
        self.rootdir_cluster
    }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        let mut path_stack = Vec::<Entry<HANDLE>>::new();
        path_stack.push(Entry::root(self.clone()));

        for component in path.as_ref().components() {
            match component {
                Component::Prefix(_) => {
                    panic!("not implemented")
                }
                Component::RootDir => {
                    path_stack.clear();
                    path_stack.push(Entry::root(self.clone()));
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    path_stack.pop();
                }
                Component::Normal(name) => {
                    use traits::Entry;
                    let top = path_stack.last().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?;
                    match top.as_dir() {
                        Some(dir) => {
                            path_stack.push(dir.find(name)?);
                        }
                        None => { return Err(io::Error::from(io::ErrorKind::InvalidInput)); }
                    }
                }
            }
        }

        Ok(path_stack.pop().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?)
    }
}
