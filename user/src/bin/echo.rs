#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;

use kernel_api::{print, println};

use crate::user::get_arguments;

mod user;

fn main() {
    for argument in get_arguments().skip(1) {
        print!("{} ", argument);
    }
    println!();
}
