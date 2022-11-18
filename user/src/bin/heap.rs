#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

mod runtime;

use alloc::string::ToString;
use core::alloc::{GlobalAlloc, Layout};
use kernel_api::syscall::{sbrk, exit};
use kernel_api::println;

fn main() {
    println!("Alloc started");

    let message = "poggers".to_string();
    println!("Message: {}", message);

    println!("Alloc finished");
}
