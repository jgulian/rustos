#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

use kernel_api::println;
use kernel_api::syscall::{duplicate, execute, fork, open, wait};

mod user;

fn main() {
    let console = open("console").expect("unable to open console");
    duplicate(console).expect("unable to duplicate console");
    duplicate(console).expect("unable to duplicate console");

    loop {
        println!("init: starting shell");
        let shell_pid = fork().expect("unable to fork");
        if shell_pid == 0 {
            execute("shell".as_bytes(), "".as_bytes())
                .expect("unable to execute shell");
        }

        while {
            let wait_pid = wait(shell_pid).expect("unable to wait for process");
            wait_pid != shell_pid
        } {}
    }
}
