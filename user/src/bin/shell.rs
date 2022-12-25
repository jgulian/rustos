#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use kernel_api::{print, println};
use kernel_api::syscall::{execute, exit, File, fork, wait};
use shim::io::{Read, Write};

mod user;

fn main() {
    let mut stdin = File::new(0);
    let mut stdout = File::new(1);
    let mut commands: Vec<String> = Vec::new();

    loop {
        let mut byte = [0u8; 1];
        let mut command = String::new();
        let mut command_index = commands.len();

        print!("shell > ");

        while {
            stdin.read(&mut byte).expect("could not read stdio");

            // TODO: work for serial escape codes broader than arrow keys
            if byte[0] == 27 {
                let mut escape_bytes = [0u8; 2];
                stdin.read(&mut escape_bytes).expect("could not read stdio");
                // up 91, 65
                // right 91, 67
                // down 91, 66
                // left 91, 68
                match escape_bytes[1] {
                    65 => {
                        match commands.get(command_index - 1) {
                            None => {}
                            Some(old_command) => {
                                print!("{}{}", "\x08 \x08".repeat(command.len()), old_command);
                                command = old_command.to_string();
                                command_index -= 1;
                            }
                        }
                    },
                    66 => {
                        match commands.get(command_index + 1) {
                            None => {
                                print!("{}", "\x08 \x08".repeat(command.len()));
                                command = "".to_string();
                                command_index = commands.len();
                            }
                            Some(old_command) => {
                                print!("{}{}", "\x08 \x08".repeat(command.len()), old_command);
                                command = old_command.to_string();
                                command_index += 1;
                            }
                        }
                    },
                    _ => {}
                }
            } else if byte[0] == 8 || byte[0] == 127 {
                match command.pop() {
                    None => {}
                    Some(_) => {
                        stdout.write("\x08 \x08".as_bytes()).expect("could not write to stdout");
                    }
                }
            } else {
                stdout.write(&byte).expect("could not write to stdout");
                command.push(byte[0] as char);
            }

            let last_char = command.chars().last().unwrap_or(' ');
            last_char != '\n' && last_char != '\r'
        } {}

        command = command.trim().to_string();
        commands.push(command.clone());
        if command.eq("exit") {
            println!();
            break;
        }

        //if let Some(command) = CommandParser::new(command).parse() {
        //    if command.run() {
        //        break;
        //    }
        //} else {
        //    println!("Invalid command");
        //}

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

//TODO: add back, though I'm not sure what it is
//enum Command {
//    Execute(Box<[u8]>, Box<[u8]>),
//    Redirect(Box<Command>, String, String),
//    Pipe(Box<Command>, Box<Command>),
//    List(Box<Command>, Box<Command>),
//}
//
//impl Command {
//    fn run(&self) -> bool {
//        match self {
//            Command::Execute(arguments, environment) => {}
//            Command::Redirect(subcommand, file) => {}
//            Command::Pipe(_, _) => {}
//            Command::List(_, _) => {}
//        }
//    }
//}
//
////TODO: use iterator probably
//struct CommandParser {
//    command: String,
//    location: usize,
//}
//
//impl CommandParser {
//    fn new(command: String) -> CommandParser {
//        CommandParser {
//            command,
//            location: 0,
//        }
//    }
//
//    fn parse(&mut self) -> Option<Command> {
//        Some(Command::Execute("".to_string(), "".to_string()))
//    }
//
//    fn peek(&self) {}
//}
//