use core::alloc::GlobalAlloc;
use core::mem::zeroed;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use core::alloc::Layout;
use core::arch::asm;
use core::prelude::rust_2024::global_allocator;

use crate::exit;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    close();
}

#[alloc_error_handler]
fn my_example_handler(layout: Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size())
}

unsafe fn zeros_bss() {
    extern "C" {
        static mut __bss_beg: u64;
        static mut __bss_end: u64;
    }

    let mut iter: *mut u64 = &mut __bss_beg;
    let end: *mut u64 = &mut __bss_end;

    while iter < end {
        write_volatile(iter, zeroed());
        iter = iter.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    asm!("svc 512");
    zeros_bss();
    crate::main();
    close();
}

fn close() -> ! {
    loop {
        let _ = exit();
    }
}

pub struct GlobalAllocator;

impl GlobalAllocator {
    const fn new() -> Self {
        GlobalAllocator {}
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        asm!("svc 420");
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
pub static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();