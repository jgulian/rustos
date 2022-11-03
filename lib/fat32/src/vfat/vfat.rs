use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;

use alloc::vec::Vec;
use core::cmp::min;
use core::{fmt, mem};

use shim::io;
use shim::path::{Component, Path};

use crate::mbr::MasterBootRecord;
use crate::{PartitionEntry, traits};
use crate::traits::{BlockDevice, FileSystem};
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
    _sectors_per_fat: u32,
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
            sector_size: bios_parameter_block.bytes_per_sector as u64,
        };

        let sectors_per_fat = if bios_parameter_block.sectors_per_fat_one != 0 {
            bios_parameter_block.sectors_per_fat_one as u32
        } else {
            bios_parameter_block.sectors_per_fat_two
        };

        let data_start_sector = bios_parameter_block.reserved_sectors as u64 +
            (bios_parameter_block.number_of_fats as u64 * sectors_per_fat as u64);

        Ok(HANDLE::new(VFat {
            phantom: Default::default(),
            device: CachedPartition::new(device, partition),
            bytes_per_sector: bios_parameter_block.bytes_per_sector,
            sectors_per_cluster: bios_parameter_block.sectors_per_cluster,
            _sectors_per_fat: sectors_per_fat,
            fat_start_sector: bios_parameter_block.reserved_sectors as u64,
            data_start_sector: data_start_sector as u64,
            rootdir_cluster: Cluster::from(bios_parameter_block.root_cluster),
        }))
    }

    pub fn read_chain(&mut self, start: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut chain_offset = ChainOffset::new(start);
        let bytes_per_cluster = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        let mut total_amount_read = 0;
        buf.resize(0, 0);
        let mut i : usize = 0;

        while !chain_offset.exhausted {
            buf.resize(buf.len() + bytes_per_cluster, 0);
            let slice = &mut buf.as_mut_slice()[(bytes_per_cluster * i)..(bytes_per_cluster * (i + 1))];
            let (amount_read, new_offset) = self.read_chain_offset(slice, chain_offset)?;

            chain_offset = new_offset;
            total_amount_read += amount_read;
            i += 1;
        }

        Ok(total_amount_read)
    }

    pub fn read_chain_offset(&mut self, buf: &mut [u8], start_offset: ChainOffset) -> io::Result<(usize, ChainOffset)> {
        let mut amount_read = 0;
        let bytes_per_cluster = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        let mut offset = start_offset.clone();

        while !offset.exhausted && amount_read != buf.len() {
            let sector = &mut buf[amount_read..];
            let n = self.read_cluster(offset.current_cluster, offset.bytes_within_cluster, sector)?;
            amount_read += n;
            offset.bytes_within_cluster = (n + offset.bytes_within_cluster) % bytes_per_cluster;

            if offset.bytes_within_cluster == 0 {
                match self.fat_entry(offset.current_cluster)?.status() {
                    Status::Eoc(_status) => {
                        offset.exhausted = true;
                    }
                    Status::Data(next_cluster) => {
                        offset.current_cluster = next_cluster;
                    }
                    _ => {
                        return Err(io::Error::from(io::ErrorKind::Other));
                    }
                }
            }
        }

        offset.total_bytes += amount_read;
        Ok((amount_read, offset))
    }

    pub fn read_cluster(&mut self, cluster: Cluster, offset: usize, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: usize = 0;
        let mut sector_id = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let last_sector = sector_id + self.sectors_per_cluster as u64;

        sector_id += offset as u64 / self.bytes_per_sector as u64;
        let mut sector_offset = offset % self.bytes_per_sector as usize;

        while amount_read < buf.len() && sector_id < last_sector {
            let buffer = self.device.get(sector_id)?;

            let amount_to_copy = min(self.bytes_per_sector as usize - sector_offset, buf.len() - amount_read);
            for i in 0..amount_to_copy {
                buf[amount_read + i] = buffer[sector_offset + i];
            }

            sector_offset = 0;
            sector_id += 1;
            amount_read += amount_to_copy;
        }

        Ok(amount_read)
    }

    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let sector = self.fat_start_sector + (cluster.offset() / self.bytes_per_sector as u32) as u64;
        let data = self.device.get(sector)?;
        let offset = (cluster.offset() % self.bytes_per_sector as u32) as usize;
        let entry = &data[offset];
        Ok(unsafe { mem::transmute::<&u8, &FatEntry>(entry) })
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

#[derive(Copy, Clone)]
pub struct ChainOffset {
    pub total_bytes: usize,
    pub bytes_within_cluster: usize,
    pub current_cluster: Cluster,
    pub exhausted: bool,
}

impl ChainOffset {
    pub(crate) fn new(start: Cluster) -> Self {
        ChainOffset {
            total_bytes: 0,
            bytes_within_cluster: 0,
            current_cluster: start,
            exhausted: false,
        }
    }
}

impl fmt::Debug for ChainOffset {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ChainOffset")
            .field("total_bytes", &self.total_bytes)
            .field("bytes_within_cluster", &self.bytes_within_cluster)
            .field("current_cluster", &self.current_cluster)
            .field("exhausted", &self.exhausted)
            .finish()
    }
}