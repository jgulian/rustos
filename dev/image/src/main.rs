mod cli;
mod device;

#[cfg(test)]
mod tests;

use std::borrow::{Borrow};
use std::fs::{File, read_dir};
use std::io;
use std::io::{copy, Error, ErrorKind, Seek, SeekFrom, Write};
use std::io::ErrorKind::InvalidInput;
use std::path::PathBuf;
use std::rc::Rc;
use clap::Parser;
use cli::ImageArgs;
use fat32::virtual_fat::{HandleReference, VirtualFat, VFatHandle};
use filesystem::fs2::FileSystem2;
use filesystem::mbr::{CHS, MasterBootRecord, PartitionEntry};
use filesystem::path::{Path};
use crate::cli::FileSystem;
use crate::cli::ImageCommand::{Create, Format};
use crate::device::ImageFile;

const BYTES_PER_MEGABYTE: u64 = 1000000;

fn main() {
    let ImageArgs {path, sector_size, command } = ImageArgs::parse();

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
    let master_boot_record = MasterBootRecord {
        bootstrap: [0; 436],
        disk_id: [0, 0, 0, 0, 199, 93, 147, 39, 0, 0],
        partition_table: [PartitionEntry {
            boot_indicator: 0,
            starting_chs: CHS { header: 0, sector: 0, cylinder: 0, },
            partition_type: 0,
            ending_chs: CHS { header: 0, sector: 0, cylinder: 0, },
            relative_sector: 0,
            total_sectors: 0,
        }; 4],
        valid_boot_sector: [85, 170],
    };

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

    master_boot_record.partition_table[partition as usize] = PartitionEntry {
        boot_indicator: 0,
        starting_chs: CHS { header: 32, sector: 8, cylinder: 0, },
        partition_type: 12,
        ending_chs: CHS { header: 143, sector: 4, cylinder: 0, },
        relative_sector: 2048,
        total_sectors: 247952,
    };

    image.seek(SeekFrom::Start(0))?;
    master_boot_record.save_writable_seekable(&mut image)?;

    let mut image_file = ImageFile::new(image, sector_size);
    let partition_entry = &mut master_boot_record.partition_table[partition as usize];
    HandleReference::<'a, BasicHandle>::format(&mut image_file, partition_entry, sector_size as usize)?;

    // TODO: obviously add more when more are supported
    let vfat_handle = VirtualFat::<BasicHandle>::from(image_file)
        .map_err(|_| Error::from(ErrorKind::Other))?;
    let mut handle_reference = HandleReference(&vfat_handle);

    add_directory_to_filesystem(&mut handle_reference, folder, filesystem::path::Path::root())?;

    Ok(())
}

#[derive(Clone, Debug)]
struct BasicHandle(Rc<VirtualFat<Self>>);

unsafe impl Sync for BasicHandle {}
unsafe impl Send for BasicHandle {}

impl VFatHandle for BasicHandle {
    fn new(val: VirtualFat<Self>) -> Self {
        BasicHandle(Rc::new(val))
    }

    fn lock<R>(&self, f: impl FnOnce(&mut VirtualFat<Self>) -> R) -> R {
        //TODO: this is to work around vfat impl; CHANGE vfat impl
        unsafe {
            let const_ptr = self.0.borrow() as *const VirtualFat<Self>;
            let mut_ptr = const_ptr as *mut VirtualFat<Self>;
            let mut_borrow = &mut *mut_ptr;
            f(mut_borrow)
        }
    }
}

fn add_directory_to_filesystem(handle_reference: &mut HandleReference<BasicHandle>, folder: PathBuf, image_path: Path) -> io::Result<()> {
    let directory_entries = read_dir(folder.clone())?;

    let mut image_directory = FileSystem2::open(handle_reference, &image_path)?
        .into_directory().ok_or(Error::from(ErrorKind::Unsupported))?;

    for directory_entry_wrapped in directory_entries {
        let directory_entry = directory_entry_wrapped?;
        let file_type = directory_entry.file_type()?;

        let entry_name_ostr = directory_entry.file_name();
        let entry_name = entry_name_ostr.to_str()
            .ok_or(Error::from(ErrorKind::Unsupported))?;

        if file_type.is_file() {
            println!("copying {}", directory_entry.path().display());
            image_directory.create_file(entry_name)?;
            let mut file = image_directory.open_entry(entry_name)?.into_file()
                .ok_or(Error::from(ErrorKind::Unsupported))?;

            let mut real_file = File::open(&directory_entry.path())?;
            copy(&mut real_file, &mut file)?;
        } else if file_type.is_dir() {
            println!("going into {}", directory_entry.path().display());
            let mut sub_folder = folder.clone();
            sub_folder.push(PathBuf::from(entry_name));

            let mut sub_image_path = image_path.clone();
            sub_image_path.append_child(entry_name);
            println!("opening {}", sub_image_path);
            image_directory.create_directory(entry_name)?;

            add_directory_to_filesystem(handle_reference, sub_folder, sub_image_path)?;
        } else {
            return Err(Error::new(InvalidInput, "Folder contains invalid file type"));
        }
    }

    Ok(())
}