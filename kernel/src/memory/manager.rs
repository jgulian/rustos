use core::sync::atomic::{AtomicUsize, Ordering};

use sync::Mutex;

use super::page_table::TranslationTable;
use crate::multiprocessing::per_core::{is_mmu_ready, set_mmu_ready};
use crate::multiprocessing::spin_lock::SpinLock;
use crate::param::{KERNEL_MASK_BITS, USER_MASK_BITS};

pub(super) struct VirtualMemoryManager {
    kernel_page_table: SpinLock<Option<TranslationTable>>,
    ready_core_count: AtomicUsize,
}

impl VirtualMemoryManager {
    pub const fn uninitialized() -> Self {
        Self {
            kernel_page_table: SpinLock::new(None),
            ready_core_count: AtomicUsize::new(0),
        }
    }

    pub fn initialize(&self) {}
}

pub struct VirtualMemoryManager2 {
    kernel_page_table: SpinLock<Option<KernPageTable>>,
    ready_core_count: AtomicUsize,
}

impl VirtualMemoryManager2 {
    /// Returns an uninitialized `VMManager`.
    ///
    /// The virtual memory manager must be initialized by calling `initialize()` and `setup()`
    /// before the first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        Self {
            kernel_page_table: SpinLock::new(None),
            ready_core_count: AtomicUsize::new(0),
        }
    }

    /// Initializes the virtual memory manager.
    /// The caller should assure that the method is invoked only once during the kernel2
    /// initialization.
    pub fn initialize(&self) {
        let kernel_page_table = KernPageTable::new();

        if self
            .kernel_page_table
            .lock(|kern_pt| kern_pt.replace(kernel_page_table))
            .unwrap()
            .is_some()
        {
            panic!("VMManager initialize called twice");
        }

        self.kern_pt_addr
            .store(base_address.as_usize(), Ordering::Relaxed);
    }

    /// Setup MMU for the current core.
    /// Wait until all cores initialize their MMU.
    pub fn wait(&self) {
        assert!(!is_mmu_ready());

        unsafe {
            self.setup();
        }

        self.ready_core_count.fetch_add(1, Ordering::Relaxed);
        while self.ready_core_count.load(Ordering::Relaxed) < pi::common::NCORES {}
    }
}
