#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(negative_impls)]
#![feature(raw_vec_internals)]
#![feature(panic_info_message)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

#[cfg(not(test))]
mod init;

extern crate alloc;
#[macro_use]
extern crate log;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod logger;
pub mod mutex;
pub mod param;
pub mod percore;
pub mod process;
pub mod shell;
pub mod traps;
pub mod vm;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Write;
use aarch64::SCR_EL3::IRQ;
use shim::path::{Path, PathBuf};
use console::kprintln;

use allocator::Allocator;
use fs::FileSystem;
use pi::interrupt::{Controller, Interrupt};
use pi::timer::tick_in;
use process::GlobalScheduler;
use traps::irq::{Fiq, GlobalIrq};
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
pub static GLOABAL_IRQ: GlobalIrq = GlobalIrq::new();
pub static FIQ: Fiq = Fiq::new();

unsafe fn kmain() -> ! {
    logger::init_logger();

    ALLOCATOR.initialize();
    FILESYSTEM.initialize();
    VMM.initialize();
    SCHEDULER.initialize();

    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    //TODO: there's an issue with locking? wherein nothing will run unless the following is here
    info!("any sussers");

    init::initialize_app_cores();
    VMM.wait();

    info!("cores initialized");

    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    SCHEDULER.start();

    kprintln!("Welcome to cs3210!");

    //SCHEDULER.start();
}
