#![feature(alloc_error_handler)]
#![feature(decl_macro)]
#![feature(const_mut_refs)]
#![feature(ptr_internals)]
#![feature(negative_impls)]
#![feature(raw_vec_internals)]
#![feature(panic_info_message)]
#![feature(is_some_and)]
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
use memory::VirtualMemoryManager2;

use traps::irq::{Fiq, GlobalIrq};

use crate::memory::KernelAllocator;
use crate::process::Process;
use crate::scheduling::{GlobalScheduler, RoundRobinScheduler};

mod init;

mod console;
mod disk;
mod memory;
mod multiprocessing;
mod param;
mod process;
mod scheduling;
mod traps;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: KernelAllocator = KernelAllocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler<RoundRobinScheduler> = GlobalScheduler::uninitialized();
pub static VIRTUAL_MEMORY: VirtualMemoryManager2 = VirtualMemoryManager2::uninitialized();
pub static GLOABAL_IRQ: GlobalIrq = GlobalIrq::new();
pub static FIQ: Fiq = Fiq::new();

unsafe fn kernel_main() -> ! {
    console::init_logger();

    ALLOCATOR.initialize();
    FILESYSTEM.initialize();
    //TODO: we want to initialize virtual memory and do the yeet as soon
    // as possible (before allocations occur). If we can get PIE we might
    // only need to worry about allocations which I think are handleable.
    VIRTUAL_MEMORY.initialize();
    SCHEDULER.initialize();

    init::initialize_app_cores();
    VIRTUAL_MEMORY.wait();

    let init = Path::try_from("/init").expect("unable to open init");
    let init_process = Process::load(&init).expect("unable to setup init");
    SCHEDULER
        .add(init_process)
        .expect("unable to add init process");

    SCHEDULER.bootstrap();
}
