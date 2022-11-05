#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

mod cr0;

use core::time::Duration;
use kernel_api::println;
use kernel_api::syscall::sleep;

fn main() {
        let elapsed = sleep(Duration::from_secs(5)).expect("");
        println!("Slept for {} millis", elapsed.as_millis());
}