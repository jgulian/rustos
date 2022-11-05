use core::mem::zeroed;
use core::panic::PanicInfo;
use core::ptr::write_volatile;
use kernel_api::syscall::exit;
use kernel_api::user_alloc::UserAllocator;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    close();
}

#[alloc_error_handler]
fn my_example_handler(layout: core::alloc::Layout) -> ! {
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
    zeros_bss();
    crate::main();
    close();
}

fn close() -> ! {
    loop {
        let _ = exit();
    }
}

#[global_allocator]
pub static USER_ALLOCATOR: UserAllocator = UserAllocator::uninitialized();