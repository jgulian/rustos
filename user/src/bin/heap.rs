#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use kernel_api::println;
use crate::user::get_arguments;

mod user;

fn main() {
    let arguments = get_arguments();
    match arguments.skip(1).next() {
        None => {}
        Some(_) => {}
    }
}
