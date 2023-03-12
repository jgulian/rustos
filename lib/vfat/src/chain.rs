use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::DerefMut;
use filesystem::device::{BlockDevice, stream_read, stream_write};
use shim::io;
use shim::io::SeekFrom;
use sync::{Mutex, MutexGuard};
use crate::cluster::Cluster;
use crate::error::{VirtualFatError, VirtualFatResult};
use crate::fat::Status;
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
        let total_size = {
            let mut lock = virtual_fat.lock().map_err(|_| VirtualFatError::FailedToLockFatMutex)?;
            (lock.block_size() * lock.fat_chain(cluster)?.len()) as u64
        };

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

    fn get_blocks(&self, guard: &mut MutexGuard<VirtualFat>) -> io::Result<impl Iterator<Item=u64>> {
        Ok(guard.fat_chain(self.current_cluster)
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?
            .into_iter().map(|cluster| Into::<u32>::into(cluster) as u64))
    }

    fn append_cluster(&self, previous: Cluster, guard: &mut MutexGuard<VirtualFat>) -> io::Result<Cluster> {
        let new_cluster = guard.next_free_cluster()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        guard.update_fat_entry(previous, Status::Data(new_cluster))
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        guard.update_fat_entry(new_cluster, Status::new_eoc())
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;
        Ok(new_cluster)
    }
}

impl io::Read for Chain {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut guard = self.virtual_fat.lock()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

        let blocks = self.get_blocks(&mut guard)?;
        let (block, read) = stream_read(guard.deref_mut(), self.position as usize, blocks, buf)?;

        self.position += read as u64;
        self.current_cluster = Cluster::from(block as u32);
        Ok(read)
    }
}

impl io::Write for Chain {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let mut guard = self.virtual_fat.lock()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

        let mut blocks: VecDeque<u64> = self.get_blocks(&mut guard)?.collect();
        let mut total_written = 0;

        while !buf.is_empty() {
            if blocks.is_empty() {
                let new_cluster = self.append_cluster(self.current_cluster, &mut guard)?;
                blocks.push_back(Into::<u32>::into(new_cluster) as u64);
            }
            let (final_block, written) =
                stream_write(guard.deref_mut(), self.position as usize, blocks.iter().map(|x| *x), buf)?;
            buf = &buf[written..];
            while let Some(block) = blocks.pop_front() {
                if block == final_block {
                    break;
                }
            }
            total_written += written;
            self.position += written as u64;
            self.current_cluster = Cluster::from(final_block as u32);
            if self.total_size < self.position {
                self.total_size = self.position;
            }
        }

        Ok(total_written)
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
                let next_cluster_wrapped = guard.next_cluster(self.current_cluster)
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
    }
}