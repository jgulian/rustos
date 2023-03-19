#[cfg(feature = "no_std")]
use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::sync::Arc;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use log::info;
use filesystem::device::{BlockDevice, stream_read, stream_write};
use shim::io;
use shim::io::SeekFrom;
use sync::Mutex;
use crate::cluster::Cluster;
use crate::error::{VirtualFatError, VirtualFatResult};
use crate::fat::Status;
use crate::virtual_fat::VirtualFat;

pub(crate) struct Chain<M: Mutex<VirtualFat>> {
    virtual_fat: Arc<M>,
    position: u64,
    total_size: u64,
    capacity: u64,
    first_cluster: Cluster,
    current_cluster: Cluster,
}

impl<M: Mutex<VirtualFat>> Chain<M> {
    pub(crate) fn new(virtual_fat: Arc<M>) -> VirtualFatResult<Self> {
        let (cluster, capacity) = virtual_fat.lock(|virtual_fat| -> VirtualFatResult<(Cluster, u64)> {
            let cluster = virtual_fat.next_free_cluster()?;
            Ok((cluster, virtual_fat.block_size() as u64))
        }).map_err(|_| VirtualFatError::FailedToLockFatMutex)??;

        Ok(Self {
            virtual_fat,
            position: 0,
            total_size: 0,
            capacity,
            first_cluster: cluster,
            current_cluster: cluster,
        })
    }

    pub(crate) fn new_from_cluster(virtual_fat: Arc<M>, cluster: Cluster) -> VirtualFatResult<Self> {
        let total_size = virtual_fat.lock(|virtual_fat| -> VirtualFatResult<u64>
            { Ok((virtual_fat.block_size() * virtual_fat.fat_chain(cluster)?.len()) as u64) })
            .map_err(|_| VirtualFatError::FailedToLockFatMutex)??;

        Ok(Self {
            virtual_fat,
            position: 0,
            total_size,
            capacity: total_size,
            first_cluster: cluster,
            current_cluster: cluster,
        })
    }

    pub(crate) fn new_from_cluster_with_size(virtual_fat: Arc<M>, cluster: Cluster, total_size: u64) -> VirtualFatResult<Self> {
        let capacity = virtual_fat.lock(|virtual_fat| -> VirtualFatResult<u64>
            { Ok((virtual_fat.block_size() * virtual_fat.fat_chain(cluster)?.len()) as u64) })
            .map_err(|_| VirtualFatError::FailedToLockFatMutex)??;

        Ok(Self {
            virtual_fat,
            position: 0,
            total_size,
            capacity,
            first_cluster: cluster,
            current_cluster: cluster,
        })
    }

    pub(crate) fn position(&self) -> u64 {
        self.position
    }

    pub(crate) fn total_size(&self) -> u64 {
        self.total_size
    }

    fn get_blocks(&self, virtual_fat: &mut VirtualFat) -> io::Result<impl Iterator<Item=u64>> {
        Ok(virtual_fat.fat_chain(self.current_cluster)
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?
            .into_iter().map(|cluster| Into::<u32>::into(cluster) as u64))
    }

    fn append_cluster(&mut self, previous: Cluster, virtual_fat: &mut VirtualFat) -> io::Result<()> {
        self.current_cluster = virtual_fat.get_clear_cluster()?;
        virtual_fat.update_fat_entry(previous, Status::Data(self.current_cluster))
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        virtual_fat.update_fat_entry(self.current_cluster, Status::new_eoc())
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        self.capacity += virtual_fat.block_size() as u64;
        Ok(())
    }
}

impl<M: Mutex<VirtualFat>> io::Read for Chain<M> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (block, read) = self.virtual_fat.lock(|virtual_fat| {
            let blocks = self.get_blocks(virtual_fat)?;
            stream_read(virtual_fat, self.position as usize, blocks, buf)
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))??;

        self.position += read as u64;
        self.current_cluster = Cluster::from(block as u32);
        Ok(read)
    }
}

impl<M: Mutex<VirtualFat>> io::Write for Chain<M> {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let virtual_fat = self.virtual_fat.clone();

        virtual_fat.lock(|virtual_fat| {
            let mut total_written = 0;

            while !buf.is_empty() {
                if self.position == self.capacity {
                    self.append_cluster(self.current_cluster, virtual_fat)?;
                }

                let blocks = self.get_blocks(virtual_fat)?;
                let real_offset = (self.position as usize) % virtual_fat.block_size();
                let (final_block, written) =
                    stream_write(virtual_fat, real_offset, blocks, buf)?;
                buf = &buf[written..];

                total_written += written;
                self.position += written as u64;
                self.current_cluster = Cluster::from(final_block as u32);
                if self.total_size < self.position {
                    self.total_size = self.position;
                }
            }

            Ok(total_written)
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))?
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl<M: Mutex<VirtualFat>> io::Seek for Chain<M> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.virtual_fat.lock(|virtual_fat| {
            let block_size = virtual_fat.block_size() as u64;

            let target_position = match pos {
                SeekFrom::Start(n) => n,
                SeekFrom::End(n) => (self.total_size as i128 + n as i128) as u64,
                SeekFrom::Current(n) => (self.position as i128 + n as i128) as u64,
            };

            loop {
                let round_position = self.position - (self.position % block_size);
                if round_position <= target_position && target_position < round_position + block_size {
                    self.position = target_position;
                    return Ok(self.position);
                } else if target_position < self.position {
                    self.position = 0;
                    self.current_cluster = self.first_cluster;
                } else {
                    let next_cluster_wrapped = virtual_fat.next_cluster(self.current_cluster)
                        .map_err(|_| io::Error::from(io::ErrorKind::Unsupported))?;

                    match next_cluster_wrapped {
                        Some(next_cluster) => {
                            self.current_cluster = next_cluster;
                            self.position += block_size - (self.position % block_size);
                        }
                        // TODO: allow seeking past end of file
                        None => return Err(io::Error::from(io::ErrorKind::Unsupported)),
                    }
                }
            }
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))?
    }
}

impl<M: Mutex<VirtualFat>> Clone for Chain<M> {
    fn clone(&self) -> Self {
        Self {
            virtual_fat: self.virtual_fat.clone(),
            position: self.position,
            total_size: self.total_size,
            capacity: self.capacity,
            first_cluster: self.first_cluster.clone(),
            current_cluster: self.current_cluster.clone(),
        }
    }
}