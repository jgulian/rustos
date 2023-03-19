#[cfg(feature = "no_std")]
use alloc::boxed::Box;
#[cfg(feature = "no_std")]
use alloc::string::String;
#[cfg(feature = "no_std")]
use alloc::sync::Arc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::boxed::Box;
#[cfg(not(feature = "no_std"))]
use std::string::String;
#[cfg(not(feature = "no_std"))]
use std::sync::Arc;
#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use core::ops::{Deref, DerefMut};
use log::info;
use shim::io;

use shim::io::SeekFrom;
use sync::Mutex;
use crate::device::{BlockDevice, ByteDevice};
use crate::filesystem::{Directory, Entry, File, Filesystem, Metadata};
use crate::master_boot_record::PartitionEntry;

use crate::path::Path;

struct Mount {
    mount_point: Path,
    filesystem: Box<dyn Filesystem>,
}

pub struct Mounts(Vec<Mount>);

pub struct VirtualFilesystem<M: Mutex<Mounts>> {
    mounts: Arc<M>,
}

impl<M: Mutex<Mounts>> VirtualFilesystem<M> {
    pub fn mount(&mut self, mount_point: Path, filesystem: Box<dyn Filesystem>) -> io::Result<()> {
        self.mounts.lock(|mounts| {
            mounts.0.push(Mount { mount_point, filesystem, });
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))
    }
}

impl<M: Mutex<Mounts>> Default for VirtualFilesystem<M> {
    fn default() -> Self {
        Self {
            mounts: Arc::new(M::new(Mounts(Vec::new()))),
        }
    }
}

impl<M: Mutex<Mounts> + 'static> Filesystem for VirtualFilesystem<M> {
    fn root(&mut self) -> io::Result<Box<dyn Directory>> {
        Ok(Box::new(VFSDirectory {
            path: Path::root(),
            mounts: self.mounts.clone(),
        }))
    }

    fn format(_: &mut dyn BlockDevice, _: &mut PartitionEntry, _: usize) -> io::Result<()> where Self: Sized {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

struct VFSDirectory<M: Mutex<Mounts>> {
    path: Path,
    mounts: Arc<M>,
}

impl<M: Mutex<Mounts> + 'static> Directory for VFSDirectory<M> {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry> {
        let mut new_path = self.path.clone();
        new_path.join_str(name)?;

        self.mounts.lock(|mounts| {
            mounts.0.iter_mut()
                .filter(|mount| mount.mount_point.starts_with(&self.path))
                .find_map(|mount| -> Option<Entry> {
                    if mount.mount_point == self.path {
                        mount.filesystem.root().ok()?.open_entry(name).ok()
                    } else if mount.mount_point.starts_with(&new_path) {
                        Some(Entry::Directory(Box::new(VFSDirectory {
                            path: new_path.clone(),
                            mounts: self.mounts.clone(),
                        })))
                    } else {
                        None
                    }
                }).ok_or(io::Error::from(io::ErrorKind::NotFound))
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))?
    }

    fn create_file(&mut self, name: &str) -> io::Result<()> {
        self.mounts.lock(|mounts| {
            mounts.0.iter_mut()
                .filter(|mount| mount.mount_point == self.path)
                .next().map(|mount| mount.filesystem.root()?.create_file(name))
                .unwrap_or(Err(io::Error::from(io::ErrorKind::Unsupported)))
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))?
    }

    fn create_directory(&mut self, _: &str) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn remove(&mut self, _: &str) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        self.mounts.deref().lock(|mounts| {
            mounts.0.iter_mut()
                .try_fold(Vec::new(), |mut result: Vec<String>, mount| {
                    match self.path.relative_from(&mount.mount_point) {
                        Some(sub_path) => {
                            let entries = mount.filesystem.open(&sub_path)
                                .and_then(|entry| entry.into_directory())
                                .map(|mut directory| directory.list())
                                .unwrap_or(Ok(Vec::new()))?;
                            result.extend(entries.into_iter());
                        }
                        None => return Err(io::Error::from(io::ErrorKind::NotFound)),
                    }

                    Ok(result)
                })
        }).map_err(|_| io::Error::from(io::ErrorKind::Other))?
    }

    fn metadata(&mut self) -> io::Result<Box<dyn Metadata>> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

pub struct ByteDeviceFilesystem<Device: ByteDevice + Send + Sync + Clone>(Device, String);
pub struct ByteDeviceDirectory<Device: ByteDevice + Send + Sync + Clone>(Device, String);
pub struct ByteDeviceFile<Device: ByteDevice + Send + Sync + Clone>(Device);

impl<Device: ByteDevice + Clone + Send + Sync + 'static> ByteDeviceFilesystem<Device> {
    pub fn new(device: Device, file: String) -> Self {
        Self(device, file)
    }
}

impl<Device: ByteDevice + Clone + Send + Sync + 'static> Filesystem for ByteDeviceFilesystem<Device> {
    fn root(&mut self) -> io::Result<Box<dyn Directory>> {
        Ok(Box::new(ByteDeviceDirectory(self.0.clone(), self.1.clone())))
    }

    fn format(device: &mut dyn BlockDevice, partition: &mut PartitionEntry, sector_size: usize) -> io::Result<()> where Self: Sized {
        todo!()
    }
}

impl<Device: ByteDevice + Clone + Send + Sync + 'static> Directory for ByteDeviceDirectory<Device> {
    fn open_entry(&mut self, name: &str) -> io::Result<Entry> {
        if self.1.eq(name) {
            Ok(Entry::File(Box::new(ByteDeviceFile(self.0.clone()))))
        } else {
            Err(io::Error::from(io::ErrorKind::NotFound))
        }
    }

    fn create_file(&mut self, name: &str) -> io::Result<()> {
        todo!()
    }

    fn create_directory(&mut self, name: &str) -> io::Result<()> {
        todo!()
    }

    fn remove(&mut self, name: &str) -> io::Result<()> {
        todo!()
    }

    fn list(&mut self) -> io::Result<Vec<String>> {
        todo!()
    }

    fn metadata(&mut self) -> io::Result<Box<dyn Metadata>> {
        todo!()
    }
}

impl<Device: ByteDevice + Clone + Send + Sync + 'static> File for ByteDeviceFile<Device> {
    fn duplicate(&mut self) -> io::Result<Box<dyn File>> {
        Ok(Box::new(ByteDeviceFile(self.0.clone())))
    }
}

impl<Device: ByteDevice + Clone + Send + Sync + 'static> io::Read for ByteDeviceFile<Device> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        io::Read::read(&mut self.0 as &mut dyn ByteDevice, buf)
    }
}

impl<Device: ByteDevice + Clone + Send + Sync + 'static> io::Write for ByteDeviceFile<Device> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::Write::write(&mut self.0 as &mut dyn ByteDevice, buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::Write::flush(&mut self.0 as &mut dyn ByteDevice)
    }
}

impl<Device: ByteDevice + Clone + Send + Sync + 'static> io::Seek for ByteDeviceFile<Device> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        io::Seek::seek(&mut self.0 as &mut dyn ByteDevice, pos)
    }
}

//impl<Device: ByteDevice + Clone + 'static> File for ByteDeviceFile<Device> {}