use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use core::arch::asm;
use core::sync::atomic::{AtomicUsize, Ordering};

use aarch64::*;
use sync::Mutex;

use crate::multiprocessing::per_core::{is_mmu_ready, set_mmu_ready};
use crate::multiprocessing::spin_lock::SpinLock;
use crate::param::{KERNEL_MASK_BITS, USER_MASK_BITS};

pub use self::address::{PhysicalAddr, VirtualAddr};
pub use self::pagetable::*;

mod address;
mod pagetable;

pub struct VMManager {
    kern_pt: SpinLock<Option<KernPageTable>>,
    kern_pt_addr: AtomicUsize,
    ready_core_cnt: AtomicUsize,
    frame_reference_counts: SpinLock<BTreeMap<PhysicalAddr, usize>>,
}

impl VMManager {
    /// Returns an uninitialized `VMManager`.
    ///
    /// The virtual memory manager must be initialized by calling `initialize()` and `setup()`
    /// before the first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        VMManager {
            kern_pt: SpinLock::new(None),
            kern_pt_addr: AtomicUsize::new(0),
            ready_core_cnt: AtomicUsize::new(0),
            frame_reference_counts: SpinLock::new(BTreeMap::new()),
        }
    }

    /// Initializes the virtual memory manager.
    /// The caller should assure that the method is invoked only once during the kernel2
    /// initialization.
    pub fn initialize(&self) {
        let kernel_page_table = KernPageTable::new();
        let base_address = kernel_page_table.get_baddr();

        if self
            .kern_pt
            .lock(|kern_pt| kern_pt.replace(kernel_page_table))
            .unwrap()
            .is_some()
        {
            panic!("VMManager initialize called twice");
        }

        self.kern_pt_addr
            .store(base_address.as_usize(), Ordering::Relaxed);
    }

    /// Set up the virtual memory manager for the current core.
    /// The caller should assure that `initialize()` has been called before calling this function.
    /// Sets proper configuration bits to MAIR_EL1, TCR_EL1, TTBR0_EL1, and TTBR1_EL1 registers.
    ///
    /// # Panics
    ///
    /// Panics if the current system does not support 64KB memory translation granule size.
    unsafe fn setup(&self) {
        assert_eq!(ID_AA64MMFR0_EL1.get_value(ID_AA64MMFR0_EL1::TGran64), 0);

        let ips = ID_AA64MMFR0_EL1.get_value(ID_AA64MMFR0_EL1::PARange);

        // (ref. D7.2.70: Memory Attribute Indirection Register)
        MAIR_EL1.set(
            0xFF |// AttrIdx=0: normal, IWBWA, OWBWA, NTR
                (0x04 << 8) |// AttrIdx=1: device, nGnRE (must be OSH too)
                (0x44 << 16), // AttrIdx=2: non cacheable
        );

        // (ref. D7.2.91: Translation Control Register)
        TCR_EL1.set(
            (ips << 32) | // IPS
                (0b11 << 30) | // TG1=64k
                (0b11 << 28) | // SH1=3 inner
                (0b01 << 26) | // ORGN1=1 write back
                (0b01 << 24) | // EPD1 enables higher half
                ((USER_MASK_BITS as u64) << 16) | // T1SZ=34 (1GB)
                (0b01 << 14) | // TG0=64k
                (0b11 << 12) | // SH0=3 inner
                (0b01 << 10) | // ORGN0=1 write back
                (0b01 << 8) | // EPD0 enables lower half
                (KERNEL_MASK_BITS as u64), // T0SZ=31 (8GB)
        );
        isb();

        let baddr = self.kern_pt_addr.load(Ordering::Relaxed);

        TTBR0_EL1.set(baddr as u64);
        TTBR1_EL1.set(baddr as u64);

        asm!("dsb ish");
        isb();

        SCTLR_EL1.set(SCTLR_EL1.get() | SCTLR_EL1::I | SCTLR_EL1::C | SCTLR_EL1::M);
        asm!("dsb sy");
        isb();

        set_mmu_ready();
    }

    /// Setup MMU for the current core.
    /// Wait until all cores initialize their MMU.
    pub fn wait(&self) {
        assert!(!is_mmu_ready());

        unsafe {
            self.setup();
        }

        //info!("MMU is ready for core-{}/@sp={:016x}", affinity(), SP.get());

        self.ready_core_cnt.fetch_add(1, Ordering::Relaxed);
        while self.ready_core_cnt.load(Ordering::Relaxed) < pi::common::NCORES {
            //info!("{} num cores ready: {}", aarch64::affinity(), self.ready_core_cnt.load(Ordering::Relaxed));
        }
    }

    /// Returns the base address of the kernel2 page table as `PhysicalAddr`.
    pub fn get_baddr(&self) -> PhysicalAddr {
        self.kern_pt
            .lock(|kern_pt| kern_pt.as_ref().unwrap().get_baddr())
            .unwrap()
    }

    pub fn pin_frame(&self, physical_address: PhysicalAddr) {
        self.frame_reference_counts
            .lock(|reference_counts| {
                *reference_counts.entry(physical_address).or_insert(0) += 1;
            })
            .unwrap()
    }

    pub fn unpin_frame(&self, physical_address: PhysicalAddr) -> bool {
        self.frame_reference_counts
            .lock(
                |reference_counts| match reference_counts.entry(physical_address) {
                    Entry::Occupied(mut e) => match *e.get() {
                        1 => {
                            e.remove();
                            true
                        }
                        x => {
                            *e.get_mut() = x - 1;
                            false
                        }
                    },
                    Entry::Vacant(_) => false,
                },
            )
            .unwrap()
    }

    pub fn get_frame_pin_count(&self, physical_address: PhysicalAddr) -> usize {
        self.frame_reference_counts
            .lock(|reference_counts| {
                reference_counts
                    .get(&physical_address)
                    .copied()
                    .unwrap_or(0usize)
            })
            .unwrap()
    }
}
