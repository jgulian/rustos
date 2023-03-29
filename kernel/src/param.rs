use core::time::Duration;

pub use pi::common::*;
use shim::const_assert_size;

// we only support 64-bit
const_assert_size!(usize, 64 / 8);

pub const USER_MASK_BITS: usize = 34;
pub const KERNEL_MASK_BITS: usize = 31;

pub const KERN_STACK_BASE: usize = 0x80_000;
pub const KERN_STACK_SIZE: usize = 0x10_000;

/// The `tick` time.
pub const TICK: Duration = Duration::from_millis(10);
