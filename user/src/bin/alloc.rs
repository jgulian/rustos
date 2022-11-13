#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

mod cr0;
mod kernel_api;

use core::ffi::CStr;
use kernel_api::{sbrk, write};
pub(crate) use kernel_api::exit;

fn main() {
    println!("Alloc started");

    let (writable_start, _) = sbrk();
    unsafe {
        let address = writable_start as *mut u8;
        for i in 0..26 {
            address.add(i).write_volatile(b'a' + i as u8);
        }
        address.add(27).write_volatile(b'\n');
        address.add(28).write_volatile(b'\0');
    };

    println!("done making alphabet in the heap");
    write(0, writable_start, 28);

    println!("Alloc finished");

    exit();
}
