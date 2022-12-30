use std::fs::File;
use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ImageArgs {
    /// The path of the disk image
    path: PathBuf,

    /// The partition to operate on
    partition: u8,

    /// The size of the sectors on the image
    #[arg(short, long, default_value_t = 512)]
    sector_size: u16,

    /// The command to run
    #[command(subcommand)]
    command: ImageCommand,
}

#[derive(Subcommand, Debug)]
enum ImageCommand {
    /// Format
    Format { filesystem: String, folder: PathBuf },
}

fn open_file(args: &ImageArgs) {
    match File::options().read(true).write(true).open(&args.path) {
        Ok(file) => {
            // TODO: check MBR
        }
        Err(_) => {}
    }
}

fn main() {
    let args = ImageArgs::parse();

    let file = File::options()
        .create_new(true)
        .read(true)
        .write(true)
        .open(args.path).expect("Unable to open file");
}