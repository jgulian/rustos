#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use kernel_api::println;

mod user;

fn main() {
    println!("Alloc started");

    let message = "poggers".to_string();
    println!("Message: {}", message);

    println!("Alloc finished");
}
