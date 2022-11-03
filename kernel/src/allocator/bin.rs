use alloc::format;
use core::alloc::Layout;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;
use crate::param::{KERN_STACK_BASE, KERN_STACK_SIZE};

const BIN_COUNT: usize = 20;

fn log_two(number: usize) -> Option<usize> {
    return if number == 0 {
        None
    } else if number == 1 {
        Some(0)
    } else {
        Some(log_two(number / 2)? + 1)
    }
}

fn is_aligned(ptr: *mut usize, align: usize) -> bool {
    align_up(ptr as usize, align) == (ptr as usize)
}

/// bin n (2^(n + 3) bytes)
pub struct Allocator {
    bins: [LinkedList; BIN_COUNT],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, mut end: usize) -> Allocator {
        let mut bins = [LinkedList::new(); BIN_COUNT];
        Allocator::allocate_bins(&mut bins, BIN_COUNT - 1, start, end);

        Allocator { bins }
    }

    fn buddy_up(&mut self, ptr: *mut usize, bin: usize) {
        let buddy = Allocator::buddy(ptr, bin);
        if bin == BIN_COUNT - 1 || !self.bins[bin].remove(buddy) {
            unsafe {
                self.bins[bin].push(ptr);
            }
            return;
        }

        let aligned_buddy = if (ptr as usize) < (buddy as usize) {
            ptr
        } else {
            buddy
        };

        self.buddy_up(aligned_buddy, bin + 1);
    }

    fn buddy_down(&mut self, bin: usize, align: usize) -> Option<*mut usize> {
        if bin >= BIN_COUNT {
            return None;
        }
        for node in self.bins[bin].iter_mut() {
            if is_aligned(node.value(), align) {
                return Some(node.pop());
            }
        }

        let buddy_above = self.buddy_down(bin + 1, align)?;
        let other_buddy = Allocator::buddy(buddy_above, bin);

        unsafe {
            self.bins[bin].push(other_buddy);
        }

        Some(buddy_above)
    }

    fn bin_size(size: usize) -> usize {
        1 << (3 + size)
    }

    fn size_to_bin(size: usize) -> Option<usize> {
        if size <= 8 {
            return Some(0);
        }
        let log = log_two(size)?;
        if size == 1 << log {
            Some(log - 3)
        } else {
            Some(log - 2)
        }
    }

    fn buddy(ptr: *mut usize, bin: usize) -> *mut usize {
        let bin_size = Allocator::bin_size(bin);
        let location = ptr as usize;
        if is_aligned(ptr, bin_size * 2) {
            (location + bin_size) as *mut usize
        } else {
            (location - bin_size) as *mut usize
        }
    }

    fn allocate_bins(bins: &mut [LinkedList; BIN_COUNT], bin: usize, start: usize, end: usize) {
        let smaller_bin = bin != 0;
        let bin_size = Allocator::bin_size(bin);
        let bin_start = align_up(start, bin_size);
        let bin_end = align_down(end, bin_size);

        if bin_end <= bin_start {
            if smaller_bin {
                Allocator::allocate_bins(bins, bin - 1, start, end);
            }
            return;
        }

        let bin_count = (bin_end - bin_start) / bin_size;
        for i in 0..bin_count {
            unsafe {
                bins[bin].push((bin_start + bin_size * i) as *mut usize);
            }
        }

        Allocator::allocate_bins(bins, bin - 1, start, bin_start);
        Allocator::allocate_bins(bins, bin - 1, bin_end, end);
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
        let bin = Allocator::size_to_bin(layout.size()).unwrap_or(0);
        self.buddy_down(bin, layout.align()).unwrap_or(ptr::null_mut()) as *mut u8
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
        let bin = Allocator::size_to_bin(layout.size()).unwrap_or(0);
        self.buddy_up(ptr as *mut usize, bin);
    }
}

// FIXME: Implement `Debug` for `Allocator`.
impl Debug for Allocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("BinAlloc()"))
    }
}
