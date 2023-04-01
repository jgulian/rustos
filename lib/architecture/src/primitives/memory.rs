use core::alloc::LayoutError;

pub trait MemoryManager {
    type MemorySegmentIterator: Iterator<Item = MemorySegment>;
    type ProcessMemory: ProcessMemory;

    fn new() -> Self
    where
        Self: Sized;

    fn memory_map() -> Self::MemorySegmentIterator;

    fn create_process_memory(&mut self) -> Self::ProcessMemory;
}

pub enum MemorySegment {
    Memory(usize, usize),
    Device(usize, usize),
}

pub trait ProcessMemory {
    type AllocationSizeIterator: Iterator<Item = usize>;

    fn allocate(&mut self, virtual_address: usize, block_size: usize);
    fn deallocate(&mut self, virtual_address: usize);
    fn translate(&mut self, virtual_address: usize) -> &mut [u8];

    fn clone(&mut self) -> Self where Self: Sized;

    fn block_sizes(&self) -> Self::AllocationSizeIterator;
}

#[derive(Debug)]
pub enum VirtualMemoryError {
    InvalidLevel,
    UnsupportedSize,
    AllocationFailed,
    DescriptorDoesNotExist,
    DescriptorOutOfBounds,
    NotPageAligned,
    AlreadyAllocated,
}

pub type VirtualMemoryResult<T> = Result<T, VirtualMemoryError>;

impl From<LayoutError> for VirtualMemoryError {
    fn from(_: LayoutError) -> Self {
        VirtualMemoryError::UnsupportedSize
    }
}
