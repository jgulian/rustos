#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(negative_impls)]
#![feature(raw_vec_internals)]
#![feature(panic_info_message)]

#![no_std]
#![no_main]
// #![cfg_attr(not(test), no_std)]
// #![cfg_attr(not(test), no_main)]

extern crate alloc;
#[macro_use]
extern crate log;




use console::kprintln;
use disk::FileSystem;


use filesystem::path::Path;
use memory::VMManager;
use process::GlobalScheduler;

use traps::irq::{Fiq, GlobalIrq};

use crate::kalloc::KernelAllocator;
use crate::process::Process;

// #[cfg(not(test))]
mod init;

pub mod kalloc;
pub mod console;
pub mod disk;
pub mod logger;
pub mod param;
pub mod process;
pub mod traps;
pub mod memory;
pub mod multiprocessing;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static GLOABAL_IRQ: GlobalIrq = GlobalIrq::new();
pub static FIQ: Fiq = Fiq::new();

unsafe fn kernel_main() -> ! {
    logger::init_logger();

    ALLOCATOR.initialize();
    FILESYSTEM.initialize();
    VMM.initialize();
    SCHEDULER.initialize();

    init::initialize_app_cores();
    VMM.wait();

    let init = Path::new("/init").expect("unable to open init");
    SCHEDULER.add(Process::load(&init).expect("unable to run init"));

    SCHEDULER.start();
}
