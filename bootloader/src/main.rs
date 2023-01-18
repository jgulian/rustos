#![feature(alloc_error_handler)]

#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::mem::zeroed;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
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

pub const KERN_STACK_BASE: usize = 0x80_000;

#[inline(never)]
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
    let core: u64;
    unsafe {
        asm!("mov sp, {}", in(reg) KERN_STACK_BASE);
        asm!("mrs {}, MPIDR_EL1", out(reg) core);
    }

    if core & 0xff != 0 {
        loop {}
    }

    initialize();
}

unsafe fn initialize() -> ! {
    zeros_bss();
    boot_main();
}

fn boot_main() -> ! {
    let mut mini_uart = MiniUart::new();

    loop {
        let byte = mini_uart.read_byte().expect("");
        mini_uart.write_byte(byte).expect("");
    }
}
