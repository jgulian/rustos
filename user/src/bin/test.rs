#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use core::cell::{RefCell, UnsafeCell};
use core::ops::Deref;
use core::time::Duration;
use kernel_api::{print, println};
use kernel_api::syscall::{exit, fork, getpid, sbrk, wait};
use kernel_api::thread::Thread;

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
        "fork_tree" => Box::new(|| {
            sbrk();
            sbrk();
            sbrk();
            fork_tree_test();
        }),
        "thread_race" => Box::new(|| {
            test_thread_will_race();
        }),
        _ => Box::new(move || {
            println!("unknown test: {}", test_name);
        }),
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

fn create_race_thread(test: Rc<UnsafeCell<i32>>) -> Thread {
    Thread::create(Box::new(move || {
        while unsafe { core::ptr::read_volatile(test.get()) == -1 } {}
        let mut count = 0;
        unsafe {
            while core::ptr::read_volatile(test.get()) < 20000 {
                count += 1;
                core::ptr::write_volatile(test.get(), core::ptr::read_volatile(test.get()) + 1);
            }
        }
        println!("Added {}", count);
    })).unwrap()
}

fn test_thread_will_race() {
    let test = Rc::new(UnsafeCell::new(-1));
    {
        let thread_a = create_race_thread(test.clone());
        let thread_b = create_race_thread(test.clone());
        println!("Starting experiment");
        unsafe { core::ptr::write_volatile(test.get(), 0) };
    }

    println!("thread race {}", unsafe { core::ptr::read_volatile(test.get()) });
}