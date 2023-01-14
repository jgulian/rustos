#![feature(alloc_error_handler)]

#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use hardware::bcm2837::uart::mini_uart::MiniUart;
use hardware::peripheral::character::CharacterDevice;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

struct PanicAllocator;

impl PanicAllocator {
    pub const fn new() -> Self {
        Self
    }
}

unsafe impl GlobalAlloc for PanicAllocator {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        panic!()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {
        panic!()
    }
}

unsafe impl Sync for PanicAllocator {}

#[global_allocator]
static ALLOCATOR: PanicAllocator = PanicAllocator::new();

#[alloc_error_handler]
pub fn alloc_handler(_: Layout) -> ! {
    panic!()
}

#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    let mut mini_uart = MiniUart::new();

    loop {
        let byte = mini_uart.read_byte().expect("");
        mini_uart.write_byte(byte).expect("");
    }
}
