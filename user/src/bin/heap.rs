#![feature(alloc_error_handler)]
#![feature(prelude_2024)]
#![no_std]
#![no_main]

extern crate alloc;

mod cr0;
mod kernel_api;

use alloc::string::ToString;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use kernel_api::{sbrk};
pub(crate) use kernel_api::exit;


struct InnerAlloc(UnsafeCell<(usize, usize)>);
unsafe impl Send for InnerAlloc {}
unsafe impl Sync for InnerAlloc {}

pub struct GlobalAllocator(InnerAlloc);

impl GlobalAllocator {
    const fn new() -> Self {
        GlobalAllocator(InnerAlloc(UnsafeCell::new((0, 0))))
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let (beg, end) = &mut *self.0.0.get();
            if *beg == 0 {
                let (alloc_beg, alloc_len) = sbrk();
                *beg = alloc_beg;
                *end = alloc_beg + alloc_len;
            }

            if *beg & (layout.align() - 1) != 0 {
                *beg = *beg & (!(layout.align() - 1)) + layout.align();
            }

            let location = unsafe {*beg as *mut u8};
            *beg += layout.size();

            location
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
pub static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();


fn main() {
    println!("Alloc started");

    let message = "poggers".to_string();
    println!("Message: {}", message);

    println!("Alloc finished");

    exit();
}
