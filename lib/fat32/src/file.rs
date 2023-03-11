
use alloc::string::String;

use filesystem;
use shim::io::{self, SeekFrom};
use crate::chain::Chain;
use crate::cluster::Cluster;
use crate::fat::Status;

#[derive(Debug, Clone)]
pub struct File {
    name: String,
    metadata: Metadata,
    file_size: u64,
    chain: Chain,
}

impl File {
    fn new(vfat: HANDLE, name: String) -> io::Result<Self> {
        let cluster = vfat.lock(|vfat| -> io::Result<Cluster> {
            let cluster = vfat.next_free_cluster()?;
            vfat.update_fat_entry(cluster, Status::new_eoc())?;
            Ok(cluster)
        })?;

        Ok(File {
            name,
            metadata: Default::default(),
            file_size: 0,
            chain: Chain::new_from_cluster(vfat.clone(), cluster)?,
        })
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.chain.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }
}


impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.chain.read(buf)
    }
}

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.chain.seek(pos)
    }
}