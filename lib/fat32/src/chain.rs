use alloc::sync::Arc;
use core::ops::DerefMut;
use filesystem::device::{BlockDevice, stream_read, stream_write};
use shim::io;
use shim::io::SeekFrom;
use sync::{Mutex, MutexGuard};
use crate::cluster::Cluster;
use crate::error::{VirtualFatError, VirtualFatResult};
use crate::virtual_fat::VirtualFat;

#[derive(Clone)]
pub(crate) struct Chain {
    virtual_fat: Arc<dyn Mutex<VirtualFat>>,
    position: u64,
    total_size: u64,
    first_cluster: Cluster,
    current_cluster: Cluster,
}

impl Chain {
    pub(crate) fn new(virtual_fat: Arc<dyn Mutex<VirtualFat>>) -> VirtualFatResult<Self> {
        let cluster = virtual_fat.lock()
            .map_err(|_| VirtualFatError::FailedToLockFatMutex)?.next_free_cluster()?;

        Ok(Self {
            virtual_fat,
            position: 0,
            total_size: 0,
            first_cluster: cluster,
            current_cluster: cluster,
        })
    }

    pub(crate) fn new_from_cluster(virtual_fat: Arc<dyn Mutex<VirtualFat>>, cluster: Cluster) -> VirtualFatResult<Self> {
        let total_size = virtual_fat.lock()
            .map_err(|_| VirtualFatError::FailedToLockFatMutex)?.fat_chain_length(cluster)?;

        Ok(Self {
            virtual_fat,
            position: 0,
            total_size,
            first_cluster: cluster,
            current_cluster: cluster,
        })
    }

    pub(crate) fn new_from_cluster_with_size(virtual_fat: Arc<dyn Mutex<VirtualFat>>, cluster: Cluster, total_size: u64) -> Self {
        Self {
            virtual_fat,
            position: 0,
            total_size,
            first_cluster: cluster,
            current_cluster: cluster,
        }
    }

    pub(crate) fn position(&self) -> u64 {
        self.position
    }

    pub(crate) fn total_size(&self) -> u64 {
        self.total_size
    }

    fn advance_cluster(&mut self, block_size: u64, guard: &mut MutexGuard<VirtualFat>) -> io::Result<bool> {
        let next_cluster_wrapped = guard.next_cluster(self.current_cluster)
            .map_err(|_| io::Error::from(io::ErrorKind::Unsupported))?;

        match next_cluster_wrapped {
            Some(next_cluster) => {
                self.current_cluster = next_cluster;
                self.position += block_size - (self.position % block_size);
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

impl io::Read for Chain {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut guard = self.virtual_fat.lock()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

        let read = stream_read(guard.deref_mut(), self.position as usize, blocks, buf)?;
        self.position += read as u64;
        Ok(read)
    }
}

impl io::Write for Chain {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.virtual_fat.lock()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

        let written = stream_write(guard.deref_mut(), self.position as usize, blocks, buf)?;
        self.position += written as u64;
        if self.total_size < self.position {
            self.total_size = self.position;
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        todo!()
    }
}

impl io::Seek for Chain {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut guard = self.virtual_fat.lock()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        let block_size = guard.block_size() as u64;

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
                self.advance_cluster(block_size, &mut guard)?;
            }
        }
    }
}