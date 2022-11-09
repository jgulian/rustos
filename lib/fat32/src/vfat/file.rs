use alloc::string::String;

use shim::io::{self, SeekFrom};

use filesystem;
use crate::vfat::{Cluster, Metadata, Status, VFatHandle};
use crate::vfat::vfat::{Chain, ChainOffset};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub name: String,
    pub metadata: Metadata,
    pub file_size: u32,
    pub(crate) chain: Chain<HANDLE>,
}

impl<HANDLE: VFatHandle> File<HANDLE> {
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

impl<HANDLE: VFatHandle> filesystem::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }

    fn size(&self) -> u64 {
        self.file_size as u64
    }
}

impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.chain.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }
}


impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.chain.read(buf)
    }
}

impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
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
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}
