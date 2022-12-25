#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

use kernel_api::println;
use kernel_api::syscall::{getpid, time};

mod user;

fn fib(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

fn main() {
    let pid: u64 = getpid().expect("unable to get pid");
    let beg = time().expect("unable to get time");
    println!("test");
    println!("[{}] Started: {}", pid, beg.as_millis());

    let rtn = fib(40);

    let end = time().expect("unable to get time");
    println!("[{}] Ended: {}", pid, end.as_millis());
    println!("[{}] Result: {} ({})", pid, rtn, (end - beg).as_millis());
}
