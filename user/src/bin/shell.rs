#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use kernel_api::{print, println};
use kernel_api::syscall::{execute, exit, File, fork, wait};
use shim::io::{Read, Write};

mod user;

fn main() {
    let mut stdin = File::new(0);
    let mut stdout = File::new(1);

    loop {
        let mut byte = [0u8; 1];
        let mut command = String::new();
        let mut escape = 0;

        print!("shell > ");

        while {
            stdin.read(&mut byte).expect("could not read stdio");

            // TODO: work for serial escape codes broader than arrow keys
            if byte[0] == 27 {
                escape = 2;
            } else if escape != 0 {
                escape -= 1;
            } else if byte[0] == 8 || byte[0] == 127 {
                stdout.write("\x08 \x08".as_bytes()).expect("could not write to stdout");
            } else {
                stdout.write(&byte).expect("could not write to stdout");
                command.push(byte[0] as char);
            }

            let last_char = command.chars().last().unwrap_or(' ');
            last_char != '\n' && last_char != '\r'
        } {}

        command = command.trim().to_string();
        if command.eq("exit") {
            println!();
            break;
        }

        let child_pid = fork().expect("could not fork");
        match child_pid {
            None => {
                println!();
                let encoded: Vec<u8> = command.chars().map(|c| {
                    match c {
                        ' ' => 0,
                        _ => c as u8,
                    }
                }).collect();
                match execute(encoded.as_slice(), "".as_ref()) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("no such command {}",
                                 command.split(' ').next().unwrap_or(""));
                        exit().expect("could not exit");
                    }
                }
            }
            Some(child) => {
                wait(child).expect("could not wait for child");
            }
        }

        command.clear();
    }
}
