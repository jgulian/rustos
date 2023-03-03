use std::path::PathBuf;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ImageArgs {
    /// The path of the disk image
    pub path: PathBuf,

    /// The size of the sectors on the image
    #[arg(short, long, default_value_t = 512)]
    pub sector_size: u16,

    /// The command to run
    #[command(subcommand)]
    pub command: ImageCommand,
}

#[derive(Subcommand, Debug)]
pub enum ImageCommand {
    /// Create a new image
    Create {
        #[arg(short, long, default_value_t = 128)]
        image_size_mb: u64,
    },
    /// Format the image by adding
    Format { filesystem: FileSystem, partition: u8, folder: PathBuf },
}

#[derive(ValueEnum, Debug, Clone)]
pub enum FileSystem {
    Fat32
}