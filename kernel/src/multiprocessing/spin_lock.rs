use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use sync::{LockError, LockResult};

use crate::multiprocessing::per_core::is_mmu_ready;

#[repr(align(32))]
pub struct SpinLock<T: Send> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
    owner: AtomicUsize,
}

unsafe impl<T: Send> Send for SpinLock<T> {}

unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T: Send> SpinLock<T> {
    pub const fn new(value: T) ->  Self {
        SpinLock {
            lock: AtomicBool::new(false),
            owner: AtomicUsize::new(usize::MAX),
            data: UnsafeCell::new(value),
        }
    }

    fn try_acquire(&self) -> LockResult<SpinLockGuard<T>> {
        let me = aarch64::affinity();

        let ordering = if is_mmu_ready() {
            Ordering::SeqCst
        } else if me == 0 {
            Ordering::Relaxed
        } else {
            return Err(LockError::InvalidState);
        };

        //TODO: Review
        if self.lock.compare_and_swap(false, true, ordering) {
            return Err(LockError::WouldBlock);
        }

        Ok(SpinLockGuard(self))
    }

    fn acquire(&self) -> LockResult<SpinLockGuard<T>> {
        loop {
            match self.try_acquire() {
                Ok(guard) => return Ok(guard),
                Err(e) if !matches!(e, LockError::WouldBlock) => {
                    return Err(e);
                }
                _ => {}
            }
        }
    }

    fn release(&self) {
        let ordering = if is_mmu_ready() {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        self.owner.store(usize::MAX, ordering);
        self.lock.store(false, ordering);
    }
}

impl<T: Send> sync::Mutex<T> for SpinLock<T> {
    fn new(value: T) -> Self where Self: Sized {
        SpinLock::new(value)
    }

    fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R> {
        let lock = self.acquire()?;
        Ok(f(unsafe {&mut *lock.0.data.get()}))
    }

    fn try_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R> {
        let lock = self.try_acquire()?;
        Ok(f(unsafe {&mut *lock.0.data.get()}))
    }

    fn is_poisoned(&self) -> bool {
        todo!()
    }

    fn clear_poison(&self) {
        todo!()
    }
}

struct SpinLockGuard<'a, T: Send>(&'a SpinLock<T>);

impl<'a, T: Send> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.0.release()
    }
}