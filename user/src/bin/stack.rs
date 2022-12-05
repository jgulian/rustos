#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

use core::arch::asm;

use kernel_api::println;
use kernel_api::syscall::write;

mod user;

fn main() {
    write(0, "STACK\n\n".as_bytes()).expect("SHEESH");
    let sp = unsafe {
        let sp: u64;
        asm!("mov {r}, sp", r = out(reg) sp);
        sp
    };

    println!("sp: {:x}", sp);

    let stack = unsafe {
        core::slice::from_raw_parts(sp as *const u8, (64 * 1024 - (sp % 64 * 1024) - 1) as usize)
    };
    println!("{:?}", stack);
}
