#[cfg(feature = "no_std")]
use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
use log::info;
#[cfg(not(feature = "no_std"))]
use std::sync::Arc;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use filesystem::filesystem;
use crate::chain::Chain;
use crate::metadata::Metadata;
use format::Format;
use shim::io;
use shim::io::{Seek, SeekFrom};
use sync::Mutex;
use crate::cluster::Cluster;
use crate::entry::{DirectoryEntry, LongFileNameEntry, parse_entry, parse_name, RegularDirectoryEntry};
use crate::virtual_fat::VirtualFat;

#[derive(Clone)]
pub(crate) struct Directory<M: Mutex<VirtualFat>> {
    pub(crate) virtual_fat: Arc<M>,
    pub(crate) metadata: Metadata,
    pub(crate) chain: Chain<M>,
}

impl<M: Mutex<VirtualFat>> Directory<M> {
    fn restart(&mut self) -> io::Result<()> {
        self.chain.seek(SeekFrom::Start(0)).map(|_| ())
    }

    //TODO: this hsould return result
    fn next(&mut self) -> io::Result<Option<DirectoryEntrySpan<M>>> {
        let start = self.chain.position();
        let mut long_file_names: Vec<LongFileNameEntry> = Vec::new();

        while self.chain.position() < self.chain.total_size() {
            let entry = DirectoryEntry::load_readable(&mut self.chain)?;
            info!("entry: {:?}", entry);

            match entry {
                DirectoryEntry::Empty => {}
                DirectoryEntry::LongFileName(lfn_entry) => {
                    long_file_names.push(lfn_entry);
                }
                DirectoryEntry::EmptyAndOver => break,
                DirectoryEntry::Regular(regular_entry) => {
                    return Ok(Some(DirectoryEntrySpan {
                        start,
                        end: self.chain.position(),
                        long_file_names,
                        regular_entry,
                        chain: &mut self.chain,
                    }));
                }
            }
        }

        Ok(None)
    }
}

impl<M: Mutex<VirtualFat> + 'static> filesystem::Directory for Directory<M> {
    fn open_entry(&mut self, name: &str) -> io::Result<filesystem::Entry> {
        self.restart()?;
        info!("restarted");
        while let Some(mut span) = self.next()? {
            // TODO: use eq?
            info!("found {}", span.name());
            if span.name().as_str() != name {
                continue;
            }

            let (starting_cluster, metadata, file_size) = span.parse_entry();

            let chain = match file_size {
                None => {
                    Chain::new_from_cluster(
                        self.virtual_fat.clone(),
                        Cluster::from(starting_cluster),
                    ).map_err(|_| io::Error::from(io::ErrorKind::Other))?
                },
                Some(file_size) =>
                    Chain::new_from_cluster_with_size(
                        self.virtual_fat.clone(),
                        Cluster::from(starting_cluster),
                        file_size as u64,
                    ),
            };

            let entry = if file_size.is_none() {
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
        while let Some(mut span) = self.next()? {
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
        while let Some(mut span) = self.next()? {
            result.push(span.name())
        }

        Ok(result)
    }

    fn metadata(&mut self) -> io::Result<Box<dyn filesystem::Metadata>> {
        Ok(Box::new(self.metadata.clone()))
    }
}

struct DirectoryEntrySpan<'a, M: Mutex<VirtualFat>> {
    start: u64,
    end: u64,
    long_file_names: Vec<LongFileNameEntry>,
    regular_entry: RegularDirectoryEntry,
    chain: &'a mut Chain<M>,
}

impl<'a, M: Mutex<VirtualFat>> DirectoryEntrySpan<'a, M> {
    fn clear(self) -> io::Result<()> {
        let count = (self.end - self.start) / 32;
        self.chain.seek(SeekFrom::Start(self.start))?;
        let empty_and_over = DirectoryEntry::EmptyAndOver;
        (0..count).try_for_each(|_| empty_and_over.save_writable(self.chain))
    }

    fn name(&mut self) -> String {
        parse_name(&mut self.long_file_names, &self.regular_entry)
    }

    fn parse_entry(&self) -> (u32, Metadata, Option<u32>) {
        parse_entry(&self.regular_entry)
    }
}