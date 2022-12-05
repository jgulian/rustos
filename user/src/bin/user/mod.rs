use core::alloc::GlobalAlloc;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::mem;
use core::mem::zeroed;
use core::panic::PanicInfo;
use core::ptr::{slice_from_raw_parts, write_volatile};

use kernel_api::println;
use kernel_api::syscall::{exit, sbrk, write};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = write(1, "panicked\n".as_bytes());
    //TODO: remove
    println!("{:?}", info);
    close();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
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

struct InnerAlloc(UnsafeCell<(usize, usize)>);

unsafe impl Send for InnerAlloc {}

unsafe impl Sync for InnerAlloc {}

pub struct GlobalAllocator(InnerAlloc);

impl GlobalAllocator {
    const fn new() -> Self {
        GlobalAllocator(InnerAlloc(UnsafeCell::new((0, 0))))
    }
}

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            let (beg, end) = &mut *self.0.0.get();
            if *beg == 0 {
                let (alloc_beg, alloc_len) = sbrk().expect("unable to alloc");
                *beg = alloc_beg;
                *end = alloc_beg + alloc_len;
            }

            if *beg & (layout.align() - 1) != 0 {
                *beg = *beg & (!(layout.align() - 1)) + layout.align();
            }

            let location = *beg as *mut u8;
            *beg += layout.size();

            location
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
pub static ALLOCATOR: GlobalAllocator = GlobalAllocator::new();

pub struct ArgumentIterator(usize, usize);

pub fn get_arguments() -> ArgumentIterator {
    let usize_size = mem::size_of::<usize>();
    let argument_count_bytes = unsafe { *((usize::MAX - usize_size + 1) as *const [u8; 8]) };
    let argument_count = usize::from_le_bytes(argument_count_bytes);
    ArgumentIterator(usize::MAX - 2 * usize_size, argument_count + 1)
}

impl Iterator for ArgumentIterator {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 == 0 {
            None
        } else {
            let mut location = self.0;
            let mut length = 1;
            while unsafe { *(location as *const u8) } != 0 {
                location -= 1;
                length += 1;
            }

            self.0 -= length;
            self.1 -= length;

            let slice = unsafe { core::slice::from_raw_parts(location as *const u8, length) };
            Some(unsafe { core::str::from_utf8_unchecked(slice) })
        }
    }
}