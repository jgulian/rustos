#![feature(alloc_error_handler)]
#![feature(decl_macro)]
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

pub mod kalloc;
pub mod console;
pub mod disk;
pub mod logger;
pub mod param;
pub mod process;
pub mod traps;
pub mod memory;
pub mod multiprocessing;

use shim::path::PathBuf;
use console::kprintln;
use filesystem::Dir;

use disk::FileSystem;
use process::GlobalScheduler;
use traps::irq::{Fiq, GlobalIrq};
use memory::VMManager;
use crate::kalloc::KernelAllocator;
use crate::process::Process;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::uninitialized();
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

    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    //TODO: there's an issue with locking? wherein nothing will run unless the following is here
    info!("any sussers");

    init::initialize_app_cores();
    VMM.wait();

    use filesystem::{FileSystem, Entry};

    info!("root dir files");
    let root = PathBuf::from("/");
    let root_dir = FILESYSTEM.open_dir(root).expect("root should exist");
    for file in root_dir.entries().expect("should be good") {
        info!("{}", file.name());
    }

    info!("cores initialized");

    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    kprintln!("Welcome to cs3210!");

    SCHEDULER.start();
}
