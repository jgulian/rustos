#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(raw_vec_internals)]
#![feature(panic_info_message)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

extern crate alloc;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod mutex;
pub mod shell;

use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Write;
use shim::path::Path;
use console::kprintln;

use allocator::Allocator;
use fs::FileSystem;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();

fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
    }

    let s = String::from("amogus");
    let root_path = Path::new("/");
    use fat32::traits::{FileSystem, Entry, Dir};
    let root = FILESYSTEM.open(root_path).expect("filesystem should have root");

    for entry in root.as_dir().expect("should have root").entries().expect("entries interator") {
        kprintln!("{}", entry.name());
    }

    kprintln!("Welcome to cs3210!");
    kprintln!("{}", s);
    shell::shell("> ");
}
