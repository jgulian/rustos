#![feature(alloc_error_handler)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::ops::Deref;
use core::time::Duration;

use kernel_api::{print, println};
use kernel_api::syscall::{exit, fork, getpid, sbrk, sleep, wait};
use kernel_api::thread::{SpinLock, Thread};
use sync::Mutex;

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
        "copy_on_write_a_lot" => Box::new(|| {
            copy_on_write_a_lot();
            println!("test over");
        }),
        "fork_tree" => Box::new(|| {
            fork_tree_test();
        }),
        "thread_race" => Box::new(|| {
            test_thread_will_race();
        }),
        "sleep_one" => Box::new(|| {
            sleep_one();
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

fn copy_on_write_a_lot() {
    for _ in 0..40 {
        sbrk().expect("unable to allocate");
    }

    let mut children = Vec::new();
    for _ in 0..100 {
        let child = fork().expect("unable to fork process");
        match child {
            None => {
                sleep(Duration::from_millis(10)).expect("unable to sleep");
            }
            Some(child) => {
                children.push(child)
            }
        }
    }

    for child in children.into_iter() {
        wait(child, None).expect("unable to wait on child");
    }
}

fn sleep_one() {
    sleep(Duration::from_secs(1)).expect("unable to sleep");
}

fn create_race_thread(data: Arc<(UnsafeCell<i32>, SpinLock<bool>)>) -> Thread {
    Thread::create(Box::new(move || {
        let (test, spin_lock) = data.deref();
        while unsafe { core::ptr::read_volatile(test.get()) == -1 } {}
        let mut count = 0;
        unsafe {
            while core::ptr::read_volatile(test.get()) < 20000 {
                count += 1;
                core::ptr::write_volatile(test.get(), core::ptr::read_volatile(test.get()) + 1);
            }
            spin_lock.lock(|_| {
                println!("Added {}", count);
            }).unwrap();
        }

    })).unwrap()
}

fn test_thread_will_race() {
    let data = Arc::new((UnsafeCell::new(-1), SpinLock::new(false)));
    {
        let thread_a = create_race_thread(data.clone());
        let thread_b = create_race_thread(data.clone());
        println!("Starting experiment");
        let (test, _) = data.deref();
        unsafe { core::ptr::write_volatile(test.get(), 0) };
    }

    let (test, _) = data.deref();
    println!("thread race {}", unsafe { core::ptr::read_volatile(test.get()) });
}

fn test_thread_safe() {}