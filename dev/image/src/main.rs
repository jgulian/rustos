mod cli;
mod device;

#[cfg(test)]
mod tests;

use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use clap::Parser;
use cli::ImageArgs;
use crate::cli::ImageCommand::{Create, Format};

const BYTES_PER_MEGABYTE: u64 = 1000000;

fn main() {
    let ImageArgs {path, sector_size, command } = ImageArgs::parse();

    match command {
        Create { image_size_mb } => {
            create_image(&path, sector_size, image_size_mb).expect("command failed");
        }
        Format { filesystem, partition, folder } => {
            
        }
    }
}

fn create_image<P: AsRef<Path>>(path: &P, sector_size: u16, image_size_mb: u64) -> io::Result<File> {
    if image_size_mb == 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "Image must have positive size"));
    }

    let file_size = image_size_mb * BYTES_PER_MEGABYTE;
    if file_size % (sector_size as u64) != 0 {
        return Err(Error::new(ErrorKind::InvalidInput, "File size must be divisible by sector size"));
    }

    let mut file = File::create(path)?;
    file.seek(SeekFrom::Start(file_size - 1))?;
    file.write(&[0])?;

    Ok(file)
}