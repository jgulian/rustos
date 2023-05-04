#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

use core::num::ParseIntError;

use kernel_api::{OsResult, println};
use kernel_api::syscall::{getpid, switch_scheduler, time};

use crate::user::get_arguments;

mod user;

fn main() {
    if let Some(argument) = get_arguments().skip(1).next() {
        match argument.parse::<usize>() {
            Ok(policy) => match switch_scheduler(policy) {
                Ok(_) => println!("successfully set policy"),
                Err(_) => println!("could not set policy"),
            },
            Err(_) => {
                println!("invalid policy name");
            }
        }
    } else {
        println!("no scheduling policy selected");
    }
}
