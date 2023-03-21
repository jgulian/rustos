#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;



use kernel_api::println;
use kernel_api::syscall::{duplicate, execute, fork, open, wait};

mod user;

fn main() {
    let console = open("/console").expect("unable to open console");
    duplicate(console, 1).expect("unable to duplicate console");
    duplicate(console, 2).expect("unable to duplicate console");

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
                    let wait_pid = wait(child_pid, None)
                        .expect("unable to wait for process").unwrap();
                    wait_pid != child_pid
                } {}
            }
        }
    }
}
