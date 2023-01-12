mod cli;

use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use clap::Parser;
use cli::ImageArgs;
use crate::cli::ImageCommand::{Create, Format};

const BYTES_PER_MEGABYTE: u64 = 1000000;

fn main() {
    let ImageArgs {path, sector_size, command } = ImageArgs::parse();

    match command {
        Create { image_size_mb } => {
            if image_size_mb == 0 {
                panic!("Image must have positive size");
            }
            let file_size = image_size_mb * BYTES_PER_MEGABYTE;
            if file_size % (sector_size as u64) == 0 {
                panic!("File size must be divisible by sector size");
            }

            let mut file = File::create(path).expect("Issue creating file");
            file.seek(SeekFrom::Start(file_size - 1)).expect("Unable to seek");
            file.write(&[0]).expect("Unable to write to file");
        }
        Format { filesystem, partition, folder } => {

        }
    }
}