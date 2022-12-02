#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use kernel_api::{print, println};
use kernel_api::syscall::{duplicate, execute, fork, getpid, open, wait};

mod user;

fn main() {
    let console = open("/console").expect("unable to open console");
    duplicate(console).expect("unable to duplicate console");
    duplicate(console).expect("unable to duplicate console");

    println!("init");
    loop {
        println!("init: starting shell");
        let shell_pid = fork().expect("unable to fork");

        match shell_pid {
            None => {
                execute("shell".as_bytes(), "".as_bytes())
                    .expect("unable to execute shell");
            }
            Some(child_pid) => {
                while {
                    let wait_pid = wait(child_pid)
                        .expect("unable to wait for process");
                    wait_pid != child_pid
                } {}
            }
        }
    }
}
