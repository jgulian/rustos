use alloc::format;
use core::alloc::Layout;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;
use crate::kprintln;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   

/// bin 0 (2^3 bytes)
/// bin 1 (2^4 bytes)
/// bin 2 (2^5 bytes)
/// bin 3 (2^6 bytes)
/// bin 4 (2^7 bytes)
/// bin 5 (2^8 bytes)
/// bin 6 (2^9 bytes)
/// bin 7 (2^10 bytes)
/// bin 8 (2^11 bytes)
/// bin 9 (2^12 bytes)
pub struct Allocator {
    linked_lists: [LinkedList; 10],
    starts: [usize; 10],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let page_size: usize = 1 << 12;
        let number_of_pages = (end - start) / page_size;
        kprintln!("number of pages: {}", number_of_pages);

        let mut linked_lists = [LinkedList::new(); 10];
        let mut starts: [usize; 10] = [0; 10];

        let mut size = end - start;
        let mut prev_start = end;

        for i in 0..10 {
            let j = 10 - i - 1;
            let bin_size = Allocator::index_to_bin_size(j);
            starts[j] = align_up(size / 2 + start, bin_size);
            size = starts[j] - start;

            let count = ((end - starts[j]) / bin_size);
            for k in 0..count {
                let l = count - k - 1;
                let address = starts[j] + bin_size * l;
                unsafe {
                    linked_lists[j].push(address as *mut usize);
                }
            }
        }

        Allocator{
            linked_lists,
            starts,
        }
    }

    fn index_to_bin_size(i: usize) -> usize {
        1 << (i + 3)
    }

    fn get_bin_index_for_size(size: usize) -> Option<usize> {
        for i in 0..10 {
            if size <= Allocator::index_to_bin_size(i) {
                return Some(i);
            }
        }

        None
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if layout.size() <= 0 || !is_power_of_two(layout.align()) {
            return ptr::null_mut();
        }

        match Allocator::get_bin_index_for_size(layout.size()) {
            None => {
                return ptr::null_mut();
            },
            Some(mut bin) => {
                while bin < 10 {
                    match self.linked_lists[bin].pop() {
                        None => continue,
                        Some(location) => {
                            return location as *mut u8;
                        }
                    }

                    bin += 1;
                }
            }
        }

        ptr::null_mut()
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut i = 0;
        let location = ptr as usize;
        while location < self.starts[i] {
            i += 1;
        }

    }

}

// FIXME: Implement `Debug` for `Allocator`.
impl Debug for Allocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("BinAlloc({:?})", self.starts))
    }
}
