use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::slice::Iter;
use filesystem::filesystem;
use crate::chain::Chain;
use crate::metadata::{Date, Metadata, Timestamp};
use format::Format;
use shim::io;
use shim::io::{Seek, SeekFrom};
use sync::Mutex;
use crate::cluster::Cluster;
use crate::entry::{DirectoryEntry, LongFileNameEntry, parse_entry, parse_name, RegularDirectoryEntry};
use crate::virtual_fat::VirtualFat;

#[derive(Clone)]
pub(crate) struct Directory {
    virtual_fat: Arc<dyn Mutex<VirtualFat>>,
    metadata: Metadata,
    chain: Chain,
}

impl Directory {
    fn restart(&mut self) -> io::Result<()> {
        self.chain.seek(SeekFrom::Start(0)).map(|_| ())
    }

    //TODO: this hsould return result
    fn next(&mut self) -> Option<DirectoryEntrySpan> {
        let mut start = 0;
        let mut long_file_names: Vec<LongFileNameEntry> = Vec::new();

        while self.chain.position() < self.chain.total_size() {
            let entry = DirectoryEntry::load_readable(&mut self.chain).ok()?;

            match entry {
                DirectoryEntry::Empty => {}
                DirectoryEntry::LongFileName(lfn_entry) => {
                    long_file_names.push(lfn_entry);
                }
                DirectoryEntry::EmptyAndOver => break,
                DirectoryEntry::Regular(regular_entry) => {
                    return Some(DirectoryEntrySpan {
                        start,
                        end: self.chain.position(),
                        long_file_names,
                        regular_entry,
                        chain: &mut self.chain,
                    });
                }
            }
        }

        None
    }
}

impl filesystem::Directory for Directory {
    fn open_entry(&mut self, name: &str) -> io::Result<filesystem::Entry> {
        self.restart()?;
        while let Some(span) = self.next() {
            if span != name {
                continue;
            }

            let (starting_cluster, metadata, directory) = span.parse_entry();

            let chain = match directory {
                None => Chain::new_from_cluster(
                    self.virtual_fat.clone(),
                    Cluster::from(starting_cluster),
                ),
                Some(file_size) =>
                    Chain::new_from_cluster_with_size(
                        self.virtual_fat.clone(),
                        Cluster::from(starting_cluster),
                        file_size as u64,
                    ),
            }.map_err(|_| io::Error::from(io::ErrorKind::Other))?;

            let entry = if directory {
                filesystem::Entry::Directory(Box::new(Directory {
                    virtual_fat: self.virtual_fat.clone(),
                    metadata,
                    chain,
                }))
            } else {
                filesystem::Entry::Directory(Box::new(Directory {
                    virtual_fat: self.virtual_fat.clone(),
                    metadata,
                    chain,
                }))
            };

            return Ok(entry);
        }

        Err(io::Error::from(io::ErrorKind::NotFound))
    }

    fn create_file(&mut self, name: &str) -> io::Result<Box<dyn filesystem::File>> {
        todo!()
    }

    fn create_directory(&mut self, name: &str) -> io::Result<Box<dyn filesystem::Directory>> {
        todo!()
    }

    fn remove(&mut self, name: &str) -> io::Result<()> {
        self.restart()?;
        while let Some(mut span) = self.next() {
            if span.name() == name {
                span.clear();
                break;
            }
        }
        Ok(())
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        self.restart()?;
        let mut result = Vec::new();
        while let Some(mut span) = self.next() {
            result.push(x.name())
        }

        Ok(result)
    }

    fn metadata(&mut self) -> io::Result<Box<dyn filesystem::Metadata>> {
        Ok(Box::new(self.metadata.clone()))
    }
}

struct DirectoryEntrySpan<'a> {
    start: u64,
    end: u64,
    long_file_names: Vec<LongFileNameEntry>,
    regular_entry: RegularDirectoryEntry,
    chain: &'a mut Chain,
}

impl<'a> DirectoryEntrySpan<'a> {
    fn clear(self) {
        let count = (self.chain.position() - start) / 32;
        self.chain.seek(SeekFrom::Start(start)).ok()?;
        let empty_and_over = DirectoryEntry::EmptyAndOver;
        (0..count).try_for_each(|_| empty_and_over.save_writable(self.chain)).ok()?;
    }

    fn name(&mut self) -> String {
        parse_name(&mut self.long_file_names, &self.regular_entry)
    }

    fn parse_entry(&self) -> (u32, Metadata, Option<u32>) {
        parse_entry(&self.regular_entry)
    }
}