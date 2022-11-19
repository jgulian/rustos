use alloc::string::String;
use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;

use alloc::vec::Vec;
use core::cmp::min;
use core::{fmt, mem};
use core::ops::DerefMut;
use log::info;

use shim::{io, ioerr, newioerr};
use shim::path::{Component, Path};

use crate::mbr::MasterBootRecord;
use crate::PartitionEntry;
use filesystem::{BlockDevice, FileSystem};
use shim::io::SeekFrom;
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
    root_cluster: Cluster,
    empty_fat_pointer: u32,
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
            sectors_per_fat,
            fat_start_sector: bios_parameter_block.reserved_sectors as u64,
            data_start_sector: data_start_sector as u64,
            root_cluster: Cluster::from(bios_parameter_block.root_cluster),
            empty_fat_pointer: 0,
        }))
    }

    fn update_sector(&mut self, sector: u64, offset: usize, buf: &[u8]) -> io::Result<()> {
        if (self.device.sector_size() as usize) < offset + buf.len() {
            panic!("attempted write out of sector");
        }

        let mut data = Vec::<u8>::new();
        let slice = if offset == 0 && buf.len() as u64 == self.device.sector_size() {
            buf
        } else {
            data.resize(self.device.sector_size() as usize, 0);
            self.device.read_sector(sector, data.as_mut_slice())?;
            (&mut data.as_mut_slice()[offset..(offset + buf.len())]).copy_from_slice(buf);
            data.as_slice()
        };

        self.device.write_sector(sector, slice)?;
        Ok(())
    }

    pub fn read_cluster(&mut self, cluster: Cluster, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: usize = 0;
        let mut sector_id = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let last_sector = sector_id + self.sectors_per_cluster as u64;

        sector_id += offset / self.bytes_per_sector as u64;
        let mut sector_offset = offset as usize % self.bytes_per_sector as usize;

        while amount_read < buf.len() && sector_id < last_sector {
            //FIXME: clean this
            let mut buffer = vec![0u8; self.device.sector_size() as usize];
            self.device.read_sector(sector_id, buffer.as_mut_slice())?;

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

    pub(crate) fn write_cluster(&mut self, cluster: Cluster, offset: u64, buf: &[u8]) -> io::Result<usize> {
        //FIXME: check buf is a good size.

        let mut amount_written = 0;
        let mut current_sector = cluster.sector_start(self.data_start_sector, self.sectors_per_cluster);
        let sector_size = self.device.sector_size();
        current_sector += offset / sector_size;

        if offset % self.device.sector_size() != 0 {
            info!("not here 1");
            let amount_to_write = (sector_size - (offset % sector_size)) as usize;
            let buffer = &buf[..amount_to_write];
            self.update_sector(current_sector, (offset % sector_size) as usize, buffer)?;
            current_sector += 1;
            amount_written += amount_to_write;
        }

        while (sector_size as usize) < buf.len() - amount_written {
            info!("not here 2");
            let buffer = &buf[amount_written..(amount_written + sector_size as usize)];
            self.device.write_sector(current_sector, buffer)?;
            current_sector += 1;
            amount_written += sector_size as usize;
        }

        if amount_written < buf.len() {
            info!("but here 3");
            let buffer = &buf[amount_written..];
            self.update_sector(current_sector, 0, buffer)?;
            amount_written += buffer.len();
        }

        Ok(amount_written)
    }

    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        let sector = self.fat_start_sector + (cluster.offset() / self.bytes_per_sector as u32) as u64;
        let mut data = vec![0u8; self.device.sector_size() as usize];
        self.device.read_sector(sector, data.as_mut_slice())?;
        let offset = (cluster.offset() % self.bytes_per_sector as u32) as usize;
        let entry = &data[offset];
        Ok(unsafe { mem::transmute::<&u8, &FatEntry>(entry) })
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
                ioerr!(Other)
            }
        }
    }

    pub fn root_cluster(&self) -> Cluster {
        self.root_cluster
    }

    pub(crate) fn next_free_cluster(&mut self) -> io::Result<Cluster> {
        let total_fat_entries = (self.sectors_per_fat as u64 * (self.device.sector_size() / mem::size_of::<FatEntry>() as u64)) as u32;
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

    pub(crate) fn bytes_per_cluster(&self) -> usize {
        (self.bytes_per_sector * self.sectors_per_cluster as u16) as usize
    }
}


//FIXME: remove exhuasted
#[derive(Clone)]
pub(crate) struct Chain<HANDLE: VFatHandle> {
    vfat: HANDLE,
    position: u64,
    first_cluster: Cluster,
    current_cluster: Cluster,
    exhausted: bool,
}

impl<HANDLE: VFatHandle> Chain<HANDLE> {
    pub(crate) fn new(vfat: HANDLE) -> io::Result<Self> {
        let cluster = vfat.lock(|vfat| {
            let cluster = vfat.next_free_cluster()?;
            vfat.update_fat_entry(cluster, Status::new_eoc())?;
            Ok(cluster)
        })?;
        Ok(Chain {
            vfat,
            position: 0,
            first_cluster: cluster,
            current_cluster: cluster,
            exhausted: false,
        })
    }

    pub(crate) fn new_from_cluster(vfat: HANDLE, cluster: Cluster) -> io::Result<Self> {
        Ok(Chain {
            vfat,
            position: 0,
            first_cluster: cluster,
            current_cluster: cluster,
            exhausted: false,
        })
    }

    pub(crate) fn position(&self) -> u64 {
        self.position
    }
}

/// Read for ChainOffset
///
/// On failure, this is an idempotent operation.
impl<HANDLE: VFatHandle> io::Read for Chain<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.vfat.lock(|vfat| {
            let bytes_per_cluster = vfat.bytes_per_cluster();
            let mut current_cluster = self.current_cluster;
            let mut cluster_offset = self.position as usize % bytes_per_cluster;
            let mut exhausted = self.exhausted;
            let mut amount_read = 0;

            while !exhausted && amount_read < buf.len() {
                let end_of_buffer = min(buf.len(), bytes_per_cluster - cluster_offset + amount_read);
                let buffer = &mut buf[amount_read..end_of_buffer];
                let read = vfat.read_cluster(current_cluster, cluster_offset as u64, buffer)?;
                amount_read += read;
                cluster_offset += read;
                //info!("in read info {} {} {}", amount_read, cluster_offset, read);
                if cluster_offset == bytes_per_cluster {
                    cluster_offset = 0;
                    match vfat.next_cluster(current_cluster)? {
                        Some(next_cluster) => {
                            current_cluster = next_cluster;
                        }
                        None => {
                            exhausted = true;
                        }
                    }
                } else if read > bytes_per_cluster {
                    panic!("read more bytes within cluster than exist within cluster");
                }
            }

            //info!("position updated by {}", amount_read);

            self.position += amount_read as u64;
            self.current_cluster = current_cluster;
            self.exhausted = exhausted;

            Ok(amount_read)
        })
    }
}

//FIXME: remove overlap with io::Read
impl<HANDLE: VFatHandle> io::Write for Chain<HANDLE> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.vfat.lock(|vfat| -> io::Result<usize> {
            let bytes_per_cluster = vfat.bytes_per_cluster();
            let mut current_cluster = self.current_cluster;
            let mut cluster_offset = self.position as usize % bytes_per_cluster;
            let mut exhausted = self.exhausted;
            let mut amount_written = 0;

            while !exhausted && amount_written < buf.len() {
                let end_of_buffer = min(buf.len(), bytes_per_cluster - cluster_offset);
                let buffer = &buf[amount_written..end_of_buffer];
                let read = vfat.write_cluster(current_cluster, cluster_offset as u64, buffer)?;
                amount_written += read;
                cluster_offset += read;
                if read == bytes_per_cluster {
                    cluster_offset = 0;
                    match vfat.next_cluster(current_cluster)? {
                        Some(next_cluster) => {
                            current_cluster = next_cluster;
                        }
                        None => {
                            exhausted = true;
                        }
                    }
                } else if read > bytes_per_cluster {
                    panic!("read more bytes within cluster than exist within cluster");
                }
            }

            info!("amogus 3 {}", amount_written);

            self.position += amount_written as u64;
            self.current_cluster = current_cluster;
            self.exhausted = exhausted;

            Ok(amount_written)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// FIXME: Just make everything a u64
// FIXME: this also doesn't allow seeking beyond the thing.
impl<HANDLE: VFatHandle> io::Seek for Chain<HANDLE> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let bytes_per_cluster = self.vfat.lock(|vfat| vfat.bytes_per_cluster()) as u64;

        match pos {
            SeekFrom::Start(n) => {
                self.exhausted = false;
                self.position = 0;
                self.current_cluster = self.first_cluster;
                self.seek(SeekFrom::Current(n as i64))
            }
            SeekFrom::End(n) => {
                loop {
                    let next_cluster = self.vfat.lock(|vfat|
                        vfat.next_cluster(self.current_cluster)
                    )?;
                    match next_cluster {
                        None => {
                            break;
                        }
                        Some(cluster) => {
                            self.current_cluster = cluster;
                        }
                    }
                    self.position += bytes_per_cluster;
                }

                self.position = (self.position / bytes_per_cluster + 1) * bytes_per_cluster - 1;
                self.seek(SeekFrom::Current(n))
            }
            SeekFrom::Current(n) => {
                if n < 0 {
                    let cluster_offset = self.position % bytes_per_cluster;
                    let target_position = self.position - ((-n) as u64);
                    if (target_position - self.position) < cluster_offset {
                        self.position = target_position;
                        Ok(self.position)
                    } else {
                        self.seek(SeekFrom::Start(target_position))
                    }
                } else if n == 0 {
                    Ok(self.position)
                } else {
                    let mut offset = n as u64;
                    while bytes_per_cluster < offset {
                        let next_cluster = self.vfat.lock(|vfat|
                            vfat.next_cluster(self.current_cluster)
                        )?;
                        match next_cluster {
                            None => {
                                return ioerr!(UnexpectedEof);
                            }
                            Some(cluster) => {
                                self.current_cluster = cluster;
                            }
                        }
                        offset -= bytes_per_cluster;
                        self.position += bytes_per_cluster;
                    }

                    self.position += offset;
                    Ok(self.position)
                }
            }
        }
    }
}

impl<HANDLE: VFatHandle> fmt::Debug for Chain<HANDLE> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("ChainOffset")
            .field("total_bytes", &self.position)
            //.field("bytes_within_cluster", &self.bytes_within_cluster)
            .field("current_cluster", &self.current_cluster)
            .field("exhausted", &self.exhausted)
            .finish()
    }
}

// FIXME: Remove this and probably most of the generics.
pub struct HandleReference<'a, HANDLE: VFatHandle>(pub &'a HANDLE);

impl<'a, HANDLE: VFatHandle> FileSystem for HandleReference<'a, HANDLE> {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open(&mut self, path: &Path) -> io::Result<Self::Entry> {
        let mut path_stack = Vec::<Entry<HANDLE>>::new();
        path_stack.push(Entry::root(self.0.clone()));

        for component in path.components() {
            match component {
                Component::Prefix(_) => {
                    panic!("not implemented")
                }
                Component::RootDir => {
                    path_stack.clear();
                    path_stack.push(Entry::root(self.0.clone()));
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    path_stack.pop();
                }
                Component::Normal(name) => {
                    use filesystem::Entry;
                    let top = path_stack.last_mut().ok_or(newioerr!(InvalidInput))?.clone();
                    let mut top_dir = top.into_dir().ok_or(newioerr!(InvalidInput))?;
                    path_stack.push(top_dir.find(name)?);
                }
            }
        }

        Ok(path_stack.pop().ok_or(io::Error::from(io::ErrorKind::InvalidInput))?)
    }

    fn new_file(&mut self, name: String) -> io::Result<Self::File> {
        Ok(File::<HANDLE> {
            name,
            metadata: Default::default(),
            file_size: 0,
            chain: Chain::new(self.0.clone())?
        })
    }

    fn new_dir(&mut self, name: String) -> io::Result<Self::Dir> {
        Ok(Dir::<HANDLE> {
            vfat: self.0.clone(),
            name,
            metadata: Default::default(),
            chain: Chain::new(self.0.clone())?
        })
    }
}