use alloc::boxed::Box;
use shim::io::{self, SeekFrom};
use sync::Mutex;
use crate::chain::Chain;
use crate::metadata::Metadata;
use crate::virtual_fat::VirtualFat;

#[derive(Clone)]
pub(crate) struct File<M: Mutex<VirtualFat>> {
    pub(crate) metadata: Metadata,
    pub(crate) file_size: u32,
    pub(crate) chain: Chain<M>,
}

impl<M: Mutex<VirtualFat> + 'static> filesystem::filesystem::File for File<M> {
    fn duplicate(&mut self) -> io::Result<Box<dyn filesystem::filesystem::File>> {
        Ok(Box::new(Self {
            metadata: self.metadata.clone(),
            file_size: self.file_size,
            chain: self.chain.clone(),
        }))
    }
}

impl<M: Mutex<VirtualFat>> io::Write for File<M> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.chain.write(buf)
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
        //TODO: set size
    }
}