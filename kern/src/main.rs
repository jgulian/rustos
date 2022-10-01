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

//extern crate alloc;

//pub mod allocator;
pub mod console;
//pub mod fs;
pub mod mutex;
pub mod shell;

use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Write;
use console::kprintln;

//use allocator::Allocator;
//use fs::FileSystem;

//#[cfg_attr(not(test), global_allocator)]
//pub static ALLOCATOR: Allocator = Allocator::uninitialized();
//pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub struct Test {}
unsafe impl GlobalAlloc for Test {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        return unsafe {0 as *mut u8}
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {

    }
}

#[global_allocator]
pub static TEST: Test = Test{};

fn kmain() -> ! {
    panic!("Here");


    //unsafe {
    //    ALLOCATOR.initialize();
    //    FILESYSTEM.initialize();
    //}
    //
    //kprintln!("Welcome to cs3210!");
    //shell::shell("> ");
}
