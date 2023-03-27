mod aligned_table;
mod allocator;
mod attributes;
mod error;
mod manager;
mod page_table;

pub use self::allocator::KernelAllocator;
pub use self::manager::VirtualMemoryManager2;