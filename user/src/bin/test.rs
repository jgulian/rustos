#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::time::Duration;
use kernel_api::{println};
use kernel_api::syscall::{exit, fork, getpid, sbrk, wait};

use crate::user::get_arguments;

mod user;

fn main() {
    for argument in get_arguments().skip(1) {
        name_to_test(argument)();
    }
}

fn name_to_test(name: &str) -> Box<dyn Fn()> {
    let test_name = name.to_string();

    match name {
        "fork_tree" => Box::new(|| { sbrk(); sbrk(); sbrk(); fork_tree_test(); }),
        _ => Box::new(move || { println!("unknown test: {}", test_name); }),
    }
}

fn unknown_test() {
    println!("unknown test")
}

fn fork_tree_test() {
    fork_tree(&mut String::new());
}

fn fork_tree(current: &mut String) {
    println!("{}: I am '{}'", getpid().expect("getpid failed"), current);
    fibonacci(30);

    fork_tree_child(current, '0');
    fork_tree_child(current, '1');
}

fn fork_tree_child(current: &mut String, branch: char) {
    if current.len() >= 3 {
        return;
    }

    current.push(branch);
    match fork().expect("fork failed") {
        None => {
            fork_tree(current);
            exit().expect("exit failed");
        }
        Some(child_pid) => {
            wait(child_pid, None).expect("wait failed");
        }
    }

    current.pop();
}

fn fibonacci(num: usize) -> usize {
    if num < 2 {
        1
    } else {
        fibonacci(num - 1) + fibonacci(num - 2)
    }
}