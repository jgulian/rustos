use core::time::Duration;

pub use pi::common::*;
use shim::const_assert_size;

// we only support 64-bit
const_assert_size!(usize, 64 / 8);

pub const PAGE_ALIGN: usize = 16;
pub const PAGE_SIZE: usize = 64 * 1024;
pub const PAGE_MASK: usize = !(PAGE_SIZE - 1);

pub const USER_MASK_BITS: usize = 34;
pub const KERNEL_MASK_BITS: usize = 31;

pub const USER_IMG_BASE: usize = 0xffff_ffff_c000_0000;

pub const KERN_STACK_BASE: usize = 0x80_000;
pub const KERN_STACK_SIZE: usize = PAGE_SIZE;

/// The `tick` time.
pub const TICK: Duration = Duration::from_secs(10);

