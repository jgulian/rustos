#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![no_std]
#![no_main]

use core::time::Duration;

use kernel_api::println;
use kernel_api::syscall::sleep;

mod user;

fn main() {
        let elapsed = sleep(Duration::from_secs(5)).expect("");
        println!("Slept for {} millis", elapsed.as_millis());
}
