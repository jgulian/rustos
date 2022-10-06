use alloc::format;
use core::alloc::Layout;
use core::fmt;
use core::fmt::{Debug, Formatter};
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;
use crate::kprintln;

const BIN_COUNT: usize = 14;

/// bin n (2^(n + 3) bytes)
pub struct Allocator {
    bins: [Bin; BIN_COUNT],
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let page_size: usize = 1 << 12;
        let number_of_pages = (end - start) / page_size;
        kprintln!("thing {} {}", start, end);
        kprintln!("memory size: {}", (end - start));
        kprintln!("number of pages: {}", number_of_pages);
        kprintln!("bin count {}", BIN_COUNT);

        let mut bins: [Bin; BIN_COUNT] = [Bin::uninitialized(); BIN_COUNT];
        let mut bin_end = end;
        for i in (0..BIN_COUNT).rev() {
            let bin_size = Allocator::bin_size(i);
            let current_bin_end = align_down(bin_end, bin_size);
            if current_bin_end < start + bin_size {
                bins[i].initialize(0, 0, bin_size);
                continue;
            }

            let bin_start = align_up((current_bin_end - start) / 2 + start, bin_size);
            if current_bin_end < bin_start + bin_size {
                bins[i].initialize(0, 0, bin_size);
                continue;
            }

            kprintln!("HERE {} {} {}", bin_start, current_bin_end, bin_size);
            bins[i].initialize(bin_start, current_bin_end, bin_size);
            bin_end = bin_start;
            kprintln!("BIN END {}", bin_end);
        }

        Allocator { bins }
    }

    fn bin_size(size: usize) -> usize {
        2 << (3 + size)
    }

    fn size_to_bin(size: usize) -> Option<usize> {
        for i in 0..BIN_COUNT {
            if size < Allocator::bin_size(i) {
                return Some(i);
            }
        }

        None
    }

    fn is_aligned(ptr: *mut usize, align: usize) -> bool {
        align_up(ptr as usize, align) == (ptr as usize)
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

        kprintln!("ALLOCATE {} {}", layout.size(), layout.align());

        match Allocator::size_to_bin(layout.size()) {
            None => {
                kprintln!("HERE 3");
            }
            Some(bin) => {
                kprintln!("ALLOCATING ON BIN {}", bin);
                let mut i = bin;
                while i < BIN_COUNT {
                    let location = self.bins[i].next(layout.align());
                    if location.is_some() {
                        return location.unwrap() as *mut u8;
                    }
                    break;
                    //i += 1;
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
        kprintln!("DEALLOC {} {}", ptr as usize, layout.size());
        let location = ptr as usize;
        let mut i = BIN_COUNT - 1;
        while location < self.bins[i].end {
            i -= 1;
        }
        if i < 0 {
            panic!("Attempted dealloc of unallocated pointer");
        }
        self.bins[i].push(ptr as *mut usize);
    }
}

// FIXME: Implement `Debug` for `Allocator`.
impl Debug for Allocator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("BinAlloc()"))
    }
}

/// Bin
#[derive(Copy, Clone)]
struct Bin {
    linked_list: LinkedList,
    start: usize,
    end: usize,
    bin_size: usize,
}

impl Bin {
    fn uninitialized() -> Bin {
        Bin { linked_list: LinkedList::new(), start: 0, end: 0, bin_size: 0 }
    }

    fn initialize(&mut self, start: usize, end: usize, bin_size: usize) {
        self.start = start;
        self.end = end;
        self.bin_size = bin_size;
    }

    fn next(&mut self, align: usize) -> Option<*mut usize> {
        if self.bin_size == 0 {
            panic!("Attempted use of uninitialized bin");
        }

        for node in self.linked_list.iter_mut() {
            if Allocator::is_aligned(node.value(), align) {
                kprintln!("FOUND {}", node.value() as usize);
                return Some(node.pop());
            }
        }

        while self.start < self.end {
            let ptr = self.start as *mut usize;
            self.start += self.bin_size;

            if Allocator::is_aligned(ptr, align) {
                kprintln!("FOUND {}", self.start);
                return Some(ptr);
            }

            unsafe {
                kprintln!("PUSHED {}", self.start);
                self.linked_list.push(self.start as *mut usize);
            }
        }
        kprintln!("HERE 1 {} {}", self.start, self.end);

        None
    }

    fn push(&mut self, ptr: *mut usize) {
        if self.bin_size == 0 {
            panic!("Attempted use of uninitialized bin");
        }

        unsafe {
            kprintln!("PUSHED BACK");
            self.linked_list.push(ptr);
        }
    }
}

