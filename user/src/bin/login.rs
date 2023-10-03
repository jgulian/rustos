#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use kernel_api::{print, println};
use kernel_api::file::File;
use kernel_api::syscall::{
    duplicate, execute, fork, get_user_identity, open, set_user_identity, wait,
};
use shim::io::{Read, Write};

mod user;

fn main() {
    let user_identity = get_user_identity().expect("unable to read user identity");
    if user_identity == 0 {
        let username = read_to_string("username");
        let password = read_to_string("password");
        println!("{} {}", username, password);

        match get_password_for_username(username.as_str()) {
            None => {
                put_password_for_username(username.as_str(), password.as_str());
                set_user_identity(user_count() + 1).expect("unable to login");
            }
            Some((stored_password, user_identity)) => {
                if password == stored_password {
                    set_user_identity(user_identity).expect("unable to login");
                } else {
                    println!("login failed; bad credentials");
                }
            }
        }
    } else {
        println!(
            "you're not root ({}); you can't change your user identity",
            user_identity
        );
    }
}

fn get_password_for_username(username: &str) -> Option<(String, u64)> {
    let mut password_file = File::new(open("/passwd").expect("could not open password file"));
    let mut data = Vec::new();
    password_file
        .read_to_end(&mut data)
        .expect("unable to read password file");
    let password_file = String::from_utf8(data).expect("unable to read password file");
    let (i, line) = password_file
        .split('\n')
        .enumerate()
        .find(|(_, user)| user.starts_with(username))?;
    Some((line.get(username.len() + 1..)?.to_string(), (i + 1) as u64))
}

fn put_password_for_username(username: &str, password: &str) {
    let mut password_file = File::new(open("/passwd").expect("could not open password file"));
    let mut data = Vec::new();
    password_file
        .read_to_end(&mut data)
        .expect("unable to read password file");
    password_file
        .write_all(username.as_bytes())
        .expect("unable to write to password file");
    password_file
        .write_all(&[b':'; 1])
        .expect("unable to write to password file");
    password_file
        .write_all(password.as_bytes())
        .expect("unable to write to password file");
    password_file
        .write_all(&[b'\n'; 1])
        .expect("unable to write to password file");
}

fn user_count() -> u64 {
    let mut password_file = File::new(open("/passwd").expect("could not open password file"));
    let mut data = Vec::new();
    password_file
        .read_to_end(&mut data)
        .expect("unable to read password file");
    (data.into_iter().filter(|c| *c == b'\n').count() + 1) as u64
}

fn read_to_string(name: &str) -> String {
    let mut stdio = File::new(0);

    print!("{}: ", name);
    let mut data = String::new();
    let mut c = [0u8; 1];
    let mut read = false;

    while {
        read = stdio.read(&mut c).expect("unable to read from stdio") == 1;
        c[0] != 13
    } {
        if read {
            data.push(c[0] as char);
            print!("{}", c[0] as char);
            read = false;
        }
    }

    println!();
    data
}
