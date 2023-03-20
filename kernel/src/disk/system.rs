use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use filesystem::path::Path;
use filesystem::pseudo::{EntryType, PseudoDirectory, PseudoDirectoryCollection, PseudoFilesystem};
use shim::io;
use crate::multiprocessing::spin_lock::SpinLock;

pub(self) struct AllocatorInformation;

impl PseudoDirectory for AllocatorInformation {
    fn read(&mut self, offset: &Path, _buf: &mut [u8]) -> io::Result<usize> {
        match offset.as_str() {
            "/allocator" => {

                Ok(0)
            }
            _ => Err(io::Error::from(io::ErrorKind::NotFound))
        }
    }

    fn write(&mut self, _: &Path, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn list(&mut self, offset: &Path) -> io::Result<Vec<String>> {
        match offset.as_str() {
            "/" => {
                let mut result = Vec::new();
                result.push(String::from("allocator"));
                Ok(result)
            }
            _ => Err(io::Error::from(io::ErrorKind::NotFound))
        }
    }

    fn entry_type(&mut self, offset: &Path) -> Option<EntryType> {
        match offset.as_str() {
            "/allocator" => Some(EntryType::File),
            _ => None,
        }
    }
}

pub(self) fn new_system_filesystem() -> io::Result<PseudoFilesystem<SpinLock<PseudoDirectoryCollection>>> {
    let mut directories: BTreeMap<Path, Box<dyn PseudoDirectory>> = BTreeMap::new();
    directories.insert(Path::root(), Box::new(AllocatorInformation));

    Ok(PseudoFilesystem::new(PseudoDirectoryCollection::new(directories)))
}