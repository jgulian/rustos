use alloc::boxed::Box;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicUsize};

use sync::{LockError, LockResult};

use crate::OsResult;
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

pub struct SpinLock<T: Send> {
    data: UnsafeCell<T>,
    held: AtomicBool,
}

impl<T: Send> sync::Mutex<T> for SpinLock<T> {
    fn new(value: T) -> Self where Self: Sized {
        Self {
            data: UnsafeCell::new(value),
            held: AtomicBool::new(false),
        }
    }

    fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R> {
        while self.held.swap(true, core::sync::atomic::Ordering::SeqCst) {}
        let result = f(unsafe { &mut *self.data.get() });
        self.held.store(false, core::sync::atomic::Ordering::SeqCst);
        Ok(result)
    }

    fn try_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R> {
        let acquired = !self.held.swap(true, core::sync::atomic::Ordering::SeqCst);
        if acquired {
            let result = f(unsafe { &mut *self.data.get() });
            self.held.store(false, core::sync::atomic::Ordering::SeqCst);
            Ok(result)
        } else {
            Err(LockError::WouldBlock)
        }
    }

    fn is_poisoned(&self) -> bool {
        false
    }

    fn clear_poison(&self) {}
}

unsafe impl<T: Send> Sync for SpinLock<T> {}