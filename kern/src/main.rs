#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(ptr_internals)]
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
pub mod param;
pub mod process;
pub mod traps;
pub mod vm;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Write;
use shim::path::{Path, PathBuf};
use console::kprintln;

use allocator::Allocator;
use fs::FileSystem;
use pi::interrupt::{Controller, Interrupt};
use pi::timer::tick_in;
use process::GlobalScheduler;
use traps::irq::Irq;
use vm::VMManager;
use pi::uart::MiniUart;
use crate::param::TICK;
use crate::process::Process;
use crate::shell::Shell;
use crate::traps::TrapFrame;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static IRQ: Irq = Irq::uninitialized();

fn run_shell() {
    Shell::new(">").run();
}

fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
        IRQ.initialize();
        SCHEDULER.initialize();
        VMM.initialize();
    }

    let mut controller = Controller::new();
    for int in Interrupt::iter() {
        controller.disable(*int);
    }

    kprintln!("Welcome to cs3210!");
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    unsafe {
        SCHEDULER.start();
    }
}
