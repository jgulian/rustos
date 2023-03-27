use core::alloc::LayoutError;

pub enum VirtualMemoryError {
    InvalidLevel,
    UnsupportedSize,
    AllocationFailed,
}

pub type VirtualMemoryResult<T> = Result<T, VirtualMemoryError>;

impl From<LayoutError> for VirtualMemoryError {
    fn from(value: LayoutError) -> Self {
        VirtualMemoryError::UnsupportedSize
    }
}