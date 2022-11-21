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

// #[cfg(not(test))]
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

use alloc::string::{String, ToString};
use core::borrow::Borrow;
use shim::path::PathBuf;
use console::kprintln;

use disk::FileSystem;
use fat32::vfat::Entry;
use filesystem::fs2::FileSystem2;
use process::GlobalScheduler;
use traps::irq::{Fiq, GlobalIrq};
use memory::VMManager;
use shim::io::{Read, Write};
use crate::kalloc::KernelAllocator;
use crate::process::Process;

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

    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    //TODO: there's an issue with locking? wherein nothing will run unless the following is here
    info!("any sussers");

    init::initialize_app_cores();
    VMM.wait();

    use filesystem::fs2::{FileSystem2, Directory2};

    info!("root dir files");

    let root_path = PathBuf::from("/");
    let mut root = FILESYSTEM.borrow().root().expect("can't turn root into dir");
    for entry in root.list().expect("can't list root") {
        info!("{}", entry);
    }

    root.create_file("amogus").expect("should allow creating files");
    //let new_file_path = PathBuf::from("/amogus");
    //let new_entry = root.open_entry("amogus").expect("can't open new file");
    //let mut new_file = new_entry.into_file().expect("can't turn new entry into file");
    //new_file.write("sussy".as_bytes()).expect("unable to write to new file");

    info!("root dir files again");
    let mut root_again = FILESYSTEM.borrow().root().expect("can't turn root into dir");
    for entry in root_again.list().expect("can't list root") {
        info!("{}", entry);
    }

    //let mut added_file = FILESYSTEM.borrow().open_file(new_file_path.as_path()).expect("amogus should exist");
    //let mut buf = [0u8; 256];
    //let _read_amount = added_file.read(&mut buf).expect("should be readable");
    // FIXME: amount read is not correct.
    //assert_eq!(read_amount, "sussy".len());
    //info!("read from new file {}", String::from_utf8_lossy(&buf));

    info!("cores initialized");

    let fib = PathBuf::from("/fib");
    let heap = PathBuf::from("/heap");

    SCHEDULER.add(Process::load(fib.as_path()).expect("should exist"));
    SCHEDULER.add(Process::load(heap.as_path()).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));
    //SCHEDULER.add(Process::load(PathBuf::from("/fib")).expect("should exist"));

    kprintln!("Welcome to rustos!");

    SCHEDULER.start();
}
