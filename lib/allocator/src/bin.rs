use core::alloc::Layout;
use core::ptr;

use crate::GenericAllocator;
use crate::linked_list::LinkedList;
use crate::util::{align_down, align_up};

const BIN_COUNT: usize = 20;

fn log_two(number: usize) -> Option<usize> {
    if number == 0 {
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
pub struct BinAllocator {
    bins: [LinkedList; BIN_COUNT],
}

impl BinAllocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Self {
        let mut bins = [LinkedList::new(); BIN_COUNT];
        Self::allocate_bins(&mut bins, BIN_COUNT - 1, start, end);

        Self { bins }
    }

    fn buddy_up(&mut self, ptr: *mut usize, bin: usize) {
        let buddy = Self::buddy(ptr, bin);
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
        let other_buddy = Self::buddy(buddy_above, bin);

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
        let bin_size = Self::bin_size(bin);
        let location = ptr as usize;
        if is_aligned(ptr, bin_size * 2) {
            (location + bin_size) as *mut usize
        } else {
            (location - bin_size) as *mut usize
        }
    }

    fn allocate_bins(bins: &mut [LinkedList; BIN_COUNT], bin: usize, start: usize, end: usize) {
        let smaller_bin = bin != 0;
        let bin_size = Self::bin_size(bin);
        let bin_start = align_up(start, bin_size);
        let bin_end = align_down(end, bin_size);

        if bin_end <= bin_start {
            if smaller_bin {
                Self::allocate_bins(bins, bin - 1, start, end);
            }
            return;
        }

        let bin_count = (bin_end - bin_start) / bin_size;
        for i in 0..bin_count {
            unsafe {
                bins[bin].push((bin_start + bin_size * i) as *mut usize);
            }
        }

        Self::allocate_bins(bins, bin - 1, start, bin_start);
        Self::allocate_bins(bins, bin - 1, bin_end, end);
    }
}

impl GenericAllocator for BinAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let bin = Self::size_to_bin(layout.size()).unwrap_or(0);
        self.buddy_down(bin, layout.align()).unwrap_or(ptr::null_mut()) as *mut u8
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let bin = Self::size_to_bin(layout.size()).unwrap_or(0);
        self.buddy_up(ptr as *mut usize, bin);
    }
}
