use core::alloc::{GlobalAlloc, Layout};
use core::ptr::drop_in_place;
use crate::ALLOCATOR;

//TODO: there has to be a better way
pub(super) struct AlignedTable<T>(*mut T, usize, Layout);

impl<T> AlignedTable<T> {
    pub(super) fn as_slice(&self) -> &[T] {
        unsafe {core::slice::from_raw_parts(self.0, self.1)}
    }

    pub(super) fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {core::slice::from_raw_parts_mut(self.0, self.1)}
    }
}

impl<T: Default> AlignedTable<T> {
    pub(super) fn new(layout: Layout) -> AlignedBoxResult<Self> {
        if layout.size() % core::mem::size_of::<T>() != 0 {
            return Err(AlignedBoxError::BadSize);
        }
        let length = layout.size() / core::mem::size_of::<T>();

        let pointer = unsafe { ALLOCATOR.alloc(layout) as *mut T };
        if pointer as usize == 0 {
            return Err(AlignedBoxError::UnableToAllocateData);
        }

        let mut result = Self(pointer, length, layout);

        {
            for element in result.as_mut_slice() {
                *element = T::default();
            }
        }

        Ok(result)
    }
}

impl<T> Drop for AlignedTable<T> {
    fn drop(&mut self) {
        unsafe {
            drop_in_place(self.as_mut_slice())
        }
    }
}

pub(super) enum AlignedBoxError {
    UnableToAllocateData,
    BadSize,
}

pub(super) type AlignedBoxResult<T> = Result<T, AlignedBoxError>;