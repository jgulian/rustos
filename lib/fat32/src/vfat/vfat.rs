use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;

use crate::mbr::MasterBootRecord;
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
        let master_boot_record = MasterBootRecord::from(device).map_err(Error::Io)?;
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

        let partition_entry = master_boot_record.partition_table[i];
        let bios_parameter_block = BiosParameterBlock::from(device, partition_entry.relative_sector)?;


        let partition = Partition{
            start: partition_entry.relative_sector,
            num_sectors: partition_entry.total_sectors,
            sector_size: partition_entry.
        };

        Ok(HANDLE::new(VFat {
            phantom: Default::default(),
            device: CachedPartition::new(device, partition),
            bytes_per_sector: bios,
            sectors_per_cluster: bios_parameter_block.sectors_per_fat,
            sectors_per_fat: bios_parameter_block.sectors_per_fat,
            fat_start_sector: 0,
            data_start_sector: 0,
            rootdir_cluster: ()
        }))
    }

    fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8],
    ) -> io::Result<usize> {
        cluster
    }

    fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>,
    ) -> io::Result<usize> {}

    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {}

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    //    fn read_cluster(
    //        &mut self,
    //        cluster: Cluster,
    //        offset: usize,
    //        buf: &mut [u8]
    //    ) -> io::Result<usize>;
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    //    fn read_chain(
    //        &mut self,
    //        start: Cluster,
    //        buf: &mut Vec<u8>
    //    ) -> io::Result<usize>;
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = crate::traits::Dummy;
    type Dir = crate::traits::Dummy;
    type Entry = crate::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }
}
