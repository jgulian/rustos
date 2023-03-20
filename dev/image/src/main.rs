mod cli;
mod logger;

#[cfg(test)]
mod tests;

use std::fs::{File, read_dir};
use std::io;
use std::io::{copy, Error, ErrorKind, Seek, SeekFrom, Write};
use std::io::ErrorKind::InvalidInput;
use std::ops::DerefMut;
use std::path::PathBuf;
use clap::Parser;
use cli::ImageArgs;
use vfat::virtual_fat::{VirtualFat, VirtualFatFilesystem};
use filesystem::filesystem::Filesystem;
use filesystem::master_boot_record::{CHS, MasterBootRecord, PartitionEntry};
use filesystem::path::{Path};
use filesystem::image::ImageFile;
use filesystem::partition::BlockPartition;
use sync::LockResult;
use crate::cli::FileSystem;
use crate::cli::ImageCommand::{Create, Format};
use log::{error, info};

const BYTES_PER_MEGABYTE: u64 = 1000000;

fn main() {
    let ImageArgs {path, sector_size, command } = ImageArgs::parse();

    logger::init_logger();

    match command {
        Create { image_size_mb } => {
            create_image(path, sector_size, image_size_mb).expect("command failed");
        }
        Format { filesystem, partition, folder } => {
            format_image(&path, sector_size, filesystem, partition, folder).expect("command failed");
        }
    }
}

fn create_image(path: PathBuf, sector_size: u16, image_size_mb: u64) -> io::Result<File> {
    if image_size_mb == 0 {
        return Err(Error::new(InvalidInput, "Image must have positive size"));
    }

    let file_size = image_size_mb * BYTES_PER_MEGABYTE;
    if file_size % (sector_size as u64) != 0 {
        return Err(Error::new(InvalidInput, "File size must be divisible by sector size"));
    }

    let mut file = File::create(path)?;
    file.seek(SeekFrom::Start(file_size - 1))?;
    file.write_all(&[0])?;

    file.seek(SeekFrom::Start(0))?;
    let master_boot_record = MasterBootRecord::default();

    use format::Format;
    master_boot_record.save_writable_seekable(&mut file)?;

    Ok(file)
}

fn format_image<'a>(path: &PathBuf, sector_size: u16, _filesystem: FileSystem, partition: u8, folder: PathBuf) -> io::Result<()> {
    if partition >= 4 {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid partition"));
    }

    use format::Format;

    let mut image = File::options().read(true).write(true).open(path)?;
    let mut master_boot_record: MasterBootRecord = MasterBootRecord::load_readable_seekable(&mut image)?;
    master_boot_record[partition as usize] = PartitionEntry {
        boot_indicator: 0,
        starting_chs: CHS { header: 32, sector: 8, cylinder: 0, },
        partition_type: 12,
        ending_chs: CHS { header: 143, sector: 4, cylinder: 0, },
        relative_sector: 2048,
        total_sectors: 247952,
    };

    image.seek(SeekFrom::Start(0))?;
    master_boot_record.save_writable_seekable(&mut image)?;

    let mut image_file = ImageFile::new(image, sector_size as usize);
    let partition_entry = &mut master_boot_record[partition as usize];
    VirtualFatFilesystem::<BasicLock<VirtualFat>>::format(&mut image_file, partition_entry, sector_size as usize)?;

    // TODO: obviously add more when more are supported
    let block_partition = BlockPartition::new(Box::new(image_file), 0xC)
        .map_err(|_| io::Error::from(io::ErrorKind::Unsupported))?;

    let mut virtual_fat_filesystem = VirtualFatFilesystem::<BasicLock<VirtualFat>>::new(block_partition)
        .map_err(|_| io::Error::from(io::ErrorKind::Unsupported))?;

    add_directory_to_filesystem(&mut virtual_fat_filesystem, folder, Path::root())?;

    Ok(())
}

fn add_directory_to_filesystem(image_filesystem: &mut dyn Filesystem, folder: PathBuf, image_path: Path) -> io::Result<()> {
    let directory_entries = read_dir(folder.clone())?;

    let mut image_directory = image_filesystem.open(&image_path)?.into_directory()?;

    for directory_entry_wrapped in directory_entries {
        let directory_entry = directory_entry_wrapped?;
        let file_type = directory_entry.file_type()?;

        let entry_name_ostr = directory_entry.file_name();
        let entry_name = entry_name_ostr.to_str()
            .ok_or(Error::from(ErrorKind::Unsupported))?;

        if file_type.is_file() {
            image_directory.create_file(entry_name)?;
            let mut file = image_directory.open_entry(entry_name)?.into_file()?;
            let mut real_file = File::open(&directory_entry.path())?;
            copy(&mut real_file, &mut file)?;
        } else if file_type.is_dir() {
            println!("going into {}", directory_entry.path().display());
            let mut sub_folder = folder.clone();
            sub_folder.push(PathBuf::from(entry_name));

            let mut sub_image_path = image_path.clone();
            sub_image_path.join_str(entry_name)?;
            println!("opening {}", sub_image_path);
            image_directory.create_directory(entry_name)?;

            add_directory_to_filesystem(image_filesystem, sub_folder, sub_image_path)?;
        } else {
            return Err(Error::new(InvalidInput, "Folder contains invalid file type"));
        }
    }

    Ok(())
}

struct BasicLock<T: Send>(std::sync::Mutex<T>);

impl<T: Send> sync::Mutex<T> for BasicLock<T> {
    fn new(value: T) -> Self where Self: Sized {
        Self(std::sync::Mutex::new(value))
    }

    fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R> {
        let mut guard = self.0.lock()
            .map_err(|_| sync::LockError::Poisoned)?;
        Ok(f(guard.deref_mut()))
    }

    fn try_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R> {
        let mut guard = self.0.lock()
            .map_err(|_| sync::LockError::Poisoned)?;
        Ok(f(guard.deref_mut()))
    }

    fn is_poisoned(&self) -> bool {
        false
    }

    fn clear_poison(&self) {}
}