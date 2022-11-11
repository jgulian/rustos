use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use log::info;
use shim::io;

use filesystem::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    virtual_sector: u64,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedPartition {
    device: Box<dyn BlockDevice>,
    cache: Vec<CacheEntry>,
    partition: Partition,
}

impl CachedPartition {
    /// Creates a new `CachedPartition` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `0` will be
    /// translated to physical sector `partition.start`. Virtual sectors of
    /// sector number `[0, num_sectors)` are accessible.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedPartition
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedPartition {
            device: Box::new(device),
            cache: Vec::new(),
            partition,
        }
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.partition.sector_size / self.device.sector_size()
    }

    /// Maps a user's request for a sector `virt` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virt: u64) -> Option<u64> {
        if virt >= self.partition.num_sectors {
            return None;
        }

        let physical_offset = virt * self.factor();
        let physical_sector = self.partition.start + physical_offset;

        Some(physical_sector)
    }

    fn cache_location(&self, virtual_sector: u64) -> Option<usize> {
        Some(self.cache.iter()
            .enumerate()
            .filter(|(i, entry)| entry.virtual_sector == virtual_sector)
            .next()?.0)
    }

    fn load_to_cache(&mut self, virtual_sector: u64) -> io::Result<usize> {
        let mut new_cache_entry = CacheEntry {
            data: vec![0; self.sector_size() as usize],
            virtual_sector,
            dirty: false
        };

        let physical_sector = self.virtual_to_physical(virtual_sector)
            .expect("the virtual sector is out of bounds");
        let device_sector_size = self.device.sector_size() as usize;

        for i in 0..self.factor() as usize {
            let slice = &mut new_cache_entry.data[device_sector_size * i..device_sector_size * (i + 1)];
            (*self.device).read_sector(physical_sector + i as u64, slice)?;
        }

        let new_index = self.cache.len();
        self.cache.push(new_cache_entry);
        Ok(new_index)
    }

    fn cache_location_or_load(&mut self, virtual_sector: u64) -> io::Result<usize> {
        let cache_location = match self.cache_location(virtual_sector) {
            None => self.load_to_cache(virtual_sector)?,
            Some(cache_location) => cache_location,
        };

        Ok(cache_location)
    }

    fn read(&mut self, virtual_sector: u64, buf: &mut [u8]) -> io::Result<()> {
        if buf.len() != self.sector_size() as usize {
            panic!("buffer can not hold sector");
        }

        let cache_location = self.cache_location_or_load(virtual_sector)?;
        buf.copy_from_slice(self.cache[cache_location].data.as_slice());
        Ok(())
    }

    fn update(&mut self, virtual_sector: u64, buf: &[u8]) -> io::Result<()> {
        if buf.len() != self.sector_size() as usize {
            //FIXME: use result
            panic!("buffer can not hold sector");
        }

        let cache_location = self.cache_location_or_load(virtual_sector)?;
        self.cache[cache_location].data.as_mut_slice().copy_from_slice(buf);
        Ok(())
    }
}

// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedPartition {
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        let slice = &mut buf[..self.sector_size() as usize];
        self.read(sector, slice)?;
        Ok(self.sector_size() as usize)
    }

    fn write_sector(&mut self, sector: u64, buf: &[u8]) -> io::Result<usize> {
        let slice = &buf[..self.sector_size() as usize];
        self.update(sector, slice)?;
        Ok(self.sector_size() as usize)
    }

    fn flush_sector(&mut self, n: u64) -> io::Result<()> {
        unimplemented!("this is not implemented")
    }
}

impl fmt::Debug for CachedPartition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedPartition")
            .field("device", &"<block device>")
            .finish()
    }
}
