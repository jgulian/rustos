use core::alloc::{GlobalAlloc, Layout};
use core::cmp::max;
use core::fmt;

use allocator::bin::BinAllocator;
use allocator::GenericAllocator;
use allocator::util::{align_down, align_up};
use pi::atags::Atags;

use crate::multiprocessing::mutex::Mutex;

/// Thread-safe (locking) wrapper around a particular memory allocator.
pub struct KernelAllocator(Mutex<Option<BinAllocator>>);

impl KernelAllocator {
    /// Returns an uninitialized `Allocator`.
    ///
    /// The allocator must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        KernelAllocator(Mutex::new(None))
    }

    /// Initializes the memory allocator.
    /// The caller should assure that the method is invoked only once during the
    /// kernel2 initialization.
    ///
    /// # Panics
    ///
    /// Panics if the system's memory map could not be retrieved.
    pub unsafe fn initialize(&self) {
        let (start, end) = memory_map().expect("failed to find memory map");
        info!("heap beg: {:x}, end: {:x}", start, end);
        *self.0.lock() = Some(BinAllocator::new(start, end));
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .as_mut()
            .expect("allocator uninitialized")
            .alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0
            .lock()
            .as_mut()
            .expect("allocator uninitialized")
            .dealloc(ptr, layout);
    }
}

extern "C" {
    static __text_end: u8;
}

/// Returns the (start address, end address) of the available memory on this
/// system if it can be determined. If it cannot, `None` is returned.
///
/// This function is expected to return `Some` under all normal cirumstances.
pub fn memory_map() -> Option<(usize, usize)> {
    let page_size = 1 << 12;
    let binary_end = unsafe { (&__text_end as *const u8) as usize };

    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let start_unaligned_address = max(mem.start as usize, binary_end);
                let start_address = align_up(start_unaligned_address, page_size);
                let end_address = align_down((mem.size + mem.start) as usize, page_size);
                return Some((start_address, end_address));
            }
            None => continue,
        }
    }

    None
}

impl fmt::Debug for KernelAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.lock().as_mut() {
            Some(_) => write!(f, "Initialized")?,
            None => write!(f, "Not yet initialized")?,
        }
        Ok(())
    }
}
