use core::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use crate::param::NCORES;
use crate::traps::irq::LocalIrq;

/// A struct to track per-core data.
#[repr(align(512))]
pub struct PerCore {
    /// Number of locks held by this core
    //preemption: AtomicI64,
    /// Is MMU initialized for this core?
    mmu_ready: AtomicBool,
    /// Local IRQ handler registry
    irq: LocalIrq,
}

static PER_CORE_DATA: [PerCore; NCORES] = [
    PerCore {
        mmu_ready: AtomicBool::new(false),
        irq: LocalIrq::new(),
    },
    PerCore {
        mmu_ready: AtomicBool::new(false),
        irq: LocalIrq::new(),
    },
    PerCore {
        mmu_ready: AtomicBool::new(false),
        irq: LocalIrq::new(),
    },
    PerCore {
        mmu_ready: AtomicBool::new(false),
        irq: LocalIrq::new(),
    },
];

/// Returns true if MMU is initialized on the current core.
pub fn is_mmu_ready() -> bool {
    let cpu = aarch64::affinity();
    PER_CORE_DATA[cpu].mmu_ready.load(Ordering::Relaxed)
}

/// Sets MMU-ready flag of the current core.
pub unsafe fn set_mmu_ready() {
    let cpu = aarch64::affinity();
    PER_CORE_DATA[cpu].mmu_ready.store(true, Ordering::Relaxed);
}

/// Returns a reference to the local IRQ handler registry of the current core.
pub fn local_irq() -> &'static LocalIrq {
    let cpu = aarch64::affinity();
    &PER_CORE_DATA[cpu].irq
}
