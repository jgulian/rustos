#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use kernel_api::{print, println};
use kernel_api::syscall::{exit, fork, getpid, wait};

use crate::user::get_arguments;

mod user;

fn main() {
    for argument in get_arguments().skip(1) {
        name_to_test(argument)();
    }
}

fn name_to_test(name: &str) -> Box<dyn Fn()> {
    let test_name = name.to_string();

    match &name[1..] {
        "fork_tree" => Box::new(|| { fork_tree_test(); }),
        _ => Box::new(move || { println!("unknown test: {}", test_name); }),
    }
}

fn unknown_test() {
    println!("unknown test")
}

fn fork_tree_test() {
    fork_tree(&mut String::new());
}

fn fork_tree(mut current: &mut String) {
    println!("{}: I am '{}'", getpid().expect("getpid failed"), current);

    fork_tree_child(&mut current, '0');
    fork_tree_child(&mut current, '1');
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
            wait(child_pid).expect("wait failed");
        }
    }

    current.pop();
}