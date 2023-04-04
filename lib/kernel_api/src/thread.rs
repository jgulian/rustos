use alloc::boxed::Box;
use core::arch::asm;
use crate::{OsResult, print, println};
use crate::syscall::{clone, exit, wait};

#[no_mangle]
unsafe extern "C" fn _create_thread() -> ! {
    let data: u64;
    asm!("mov {}, x0", out(reg) data);
    let function = unsafe {
        *Box::from_raw(data as usize as *mut Box<dyn FnOnce()>)
    };

    function();

    exit().expect("unable to exit thread");
    panic!("unable to exit thread");
}

pub struct Thread(u64);

impl Thread {
    pub fn create(function: Box<dyn FnOnce()>) -> OsResult<Thread> {
        // TODO: this definitely isn't safe lol... or rather, this is safe, but it may cause
        // unwanted side effects.
        let boxed_box = Box::new(function);
        let function_pointer = Box::into_raw(boxed_box) as usize;
        let pid = clone(_create_thread as usize, function_pointer)?;
        Ok(Thread(pid))
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        wait(self.0, None).expect("unable to join thread");
    }
}
