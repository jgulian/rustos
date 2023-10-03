#![no_std]

use crate::statistics::AllocatorStatistics;
use core::alloc::Layout;

pub mod bin;
pub mod linked_list;
pub mod statistics;
pub mod util;

#[cfg(test)]
pub mod tests;

pub trait GenericAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
    fn statistics(&mut self) -> AllocatorStatistics;
}
