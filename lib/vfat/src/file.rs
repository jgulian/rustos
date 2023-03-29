use crate::chain::Chain;
use crate::directory::Directory;
use crate::metadata::Metadata;
use crate::virtual_fat::VirtualFat;
#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::string::String;
use log::info;
use shim::io::{self, SeekFrom};
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::string::String;
use sync::Mutex;

#[derive(Clone)]
pub(crate) struct File<M: Mutex<VirtualFat>> {
    pub(crate) name: String,
    pub(crate) directory: Directory<M>,
    pub(crate) metadata: Metadata,
    pub(crate) file_size: u32,
    pub(crate) chain: Chain<M>,
}

impl<M: Mutex<VirtualFat> + 'static> filesystem::filesystem::File for File<M> {
    fn duplicate(&mut self) -> io::Result<Box<dyn filesystem::filesystem::File>> {
        Ok(Box::new(Self {
            name: self.name.clone(),
            directory: self.directory.clone(),
            metadata: self.metadata.clone(),
            file_size: self.file_size,
            chain: self.chain.clone(),
        }))
    }
}

impl<M: Mutex<VirtualFat>> io::Write for File<M> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.chain.write(buf)?;
        if self.file_size < self.chain.total_size() as u32 {
            self.file_size = self.chain.total_size() as u32;
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("not required")
    }
}

impl<M: Mutex<VirtualFat>> io::Read for File<M> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.chain.read(buf)
    }
}

impl<M: Mutex<VirtualFat>> io::Seek for File<M> {
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

impl<M: Mutex<VirtualFat>> Drop for File<M> {
    fn drop(&mut self) {
        self.directory
            .update_file_size(self.name.as_str(), self.file_size)
            .expect("unable to update file size");
    }
}
