use core::arch::asm;

/// Wait for event not to burn CPU.
#[inline(always)]
pub fn wfe() {
    unsafe { asm!("wfe") };
}

/// Wait for interrupt not to burn CPU.
#[inline(always)]
pub fn wfi() {
    unsafe { asm!("wfi") };
}

/// A NOOP that won't be optimized out.
#[inline(always)]
pub fn nop() {
    unsafe { asm!("nop") };
}

/// Transition to a lower level
#[inline(always)]
pub unsafe fn eret() {
    asm!("eret");
}

/// Instruction Synchronization Barrier
#[inline(always)]
pub fn isb() {
    unsafe { asm!("isb") };
}

/// Set Event
#[inline(always)]
pub fn sev() {
    unsafe { asm!("sev") };
}

/// Enable (unmask) interrupts
#[inline(always)]
pub fn enable_irq_interrupt() {
    unsafe {
        asm!("msr DAIFClr, 0b0010");
    }
}

/// Disable (mask) interrupt
#[inline(always)]
pub fn disable_irq_interrupt() {
    unsafe {
        asm!("msr DAIFSet, 0b0010");
    }
}

/// Enable (unmask) FIQ
#[inline(always)]
pub fn enable_fiq_interrupt() {
    unsafe {
        asm!("msr DAIFClr, 0b0001");
    }
}

/// Disable (mask) FIQ
#[inline(always)]
pub fn disable_fiq_interrupt() {
    unsafe {
        asm!("msr DAIFSet, 0b0001");
    }
}

pub fn get_interrupt_mask() -> u64 {
    unsafe {
        let mut mask: u64;
        asm!("mrs {r}, DAIF",
        r = out(reg) mask,
        );
        mask
    }
}

pub fn set_interrupt_mask(mask: u64) {
    unsafe {
        asm!("msr DAIF, {s}",
        s = in(reg) mask,
        );
    }
}

// FIXME: these should be unsafe