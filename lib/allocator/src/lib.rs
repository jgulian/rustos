#![no_std]

use core::alloc::Layout;
use crate::statistics::AllocatorStatistics;

pub mod bin;
pub mod linked_list;
pub mod util;
pub mod statistics;

#[cfg(test)]
pub mod tests;

pub trait GenericAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
    fn statistics(&mut self) -> AllocatorStatistics;
}
