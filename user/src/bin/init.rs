#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;

use kernel_api::{print, println};
use kernel_api::syscall::{duplicate, execute, fork, open, wait};

mod user;

fn main() {
    let console = open("/console").expect("unable to open console");
    duplicate(console).expect("unable to duplicate console");
    duplicate(console).expect("unable to duplicate console");

    println!("init");
    loop {
        println!("init: starting shell");
        let shell_pid_wrapped = fork(); //.expect("unable to fork");
        println!("shell_pid {}", shell_pid_wrapped.map(|r| r.to_string()).unwrap_or("err".to_string()));

        let shell_pid = shell_pid_wrapped.expect("unable to fork");
        if shell_pid == 0 {
            println!("here 1");
            execute("shell".as_bytes(), "".as_bytes())
                .expect("unable to execute shell");
        }
        println!("here 2");

        while {
            let wait_pid = wait(shell_pid).expect("unable to wait for process");
            println!("done waiting");
            wait_pid != shell_pid
        } {}
        println!("here 3")
    }
}
