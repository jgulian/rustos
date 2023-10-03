use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::min;

use filesystem::path::Path;
use filesystem::pseudo::{EntryType, PseudoDirectory, PseudoDirectoryCollection, PseudoFilesystem};
use shim::io;

use crate::ALLOCATOR;
use crate::multiprocessing::spin_lock::SpinLock;

struct AllocatorInformation;

impl PseudoDirectory for AllocatorInformation {
    fn read(&mut self, offset: &Path, buf: &mut [u8]) -> io::Result<usize> {
        match offset.as_str() {
            "allocator" => {
                let statistics = ALLOCATOR.stats();
                let data = format!(
                    "allocated_size: {}\nallocation_count: {}\ntotal_memory: {}\n",
                    statistics.allocated_size, statistics.allocation_count, statistics.total_memory
                );

                let len = min(data.len(), buf.len());
                let bytes = &data.as_bytes()[..len];
                buf[..len].copy_from_slice(bytes);
                Ok(len)
            }
            _ => Err(io::Error::from(io::ErrorKind::NotFound)),
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
            _ => Err(io::Error::from(io::ErrorKind::NotFound)),
        }
    }

    fn entry_type(&mut self, offset: &Path) -> Option<EntryType> {
        match offset.as_str() {
            "allocator" => Some(EntryType::File),
            _ => None,
        }
    }
}

pub(super) fn new_system_filesystem(
) -> io::Result<PseudoFilesystem<SpinLock<PseudoDirectoryCollection>>> {
    let mut directories: BTreeMap<Path, Box<dyn PseudoDirectory>> = BTreeMap::new();
    directories.insert(Path::try_from("allocator")?, Box::new(AllocatorInformation));

    Ok(PseudoFilesystem::new(PseudoDirectoryCollection::new(
        directories,
    )))
}
