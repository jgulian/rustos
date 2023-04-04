#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use kernel_api::file::File;
use kernel_api::println;
use kernel_api::syscall::{open, write};
use shim::io::Read;

use crate::user::get_arguments;

mod user;

fn cat(mut file: File) {
    let mut data = [0u8; 128];
    while {
        match file.read(&mut data) {
            Ok(n) => {
                let _ = write(1, &data[..n]);
                n == 128
            }
            Err(e) => {
                println!("unable to read file {:?}", e);
                false
            }
        }
    } {}
}

fn main() {
    match get_arguments().nth(1) {
        Some(file) => match open(file.trim_matches(0 as char)) {
            Ok(id) => {
                cat(File::new(id));
            }
            Err(_) => {
                println!("Unable to open file");
            }
        },
        None => cat(File::new(0)),
    }
}
