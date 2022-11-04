#![no_std]

use core::alloc::{Layout};

pub mod bin;
pub mod linked_list;
pub mod util;

#[cfg(test)]
pub mod tests;

pub trait GenericAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
}
