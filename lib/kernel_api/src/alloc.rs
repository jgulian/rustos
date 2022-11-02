use core::alloc::{GlobalAlloc, Layout};

use super::syscall::sbrk;

// FIXME: use another allocator, maybe adapt bin-buddy

pub struct UserAllocator {
}

impl UserAllocator {
    pub const fn uninitialized() -> Self {
        UserAllocator {
        }
    }
}

unsafe impl GlobalAlloc for UserAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        unimplemented!("this is not implemented yet");
    }


    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}