use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use sync::{LockResult, TryLockError, TryLockResult};

use crate::multiprocessing::per_core::is_mmu_ready;

#[repr(align(32))]
pub struct SpinLock<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
    owner: AtomicUsize,
}

unsafe impl<T: Send> Send for SpinLock<T> {}

unsafe impl<T: Send> Sync for SpinLock<T> {}

struct SpinLockMutexGuard<'a, T>(&'a SpinLock<T>);

impl<'a, T> sync::Mutex<T> for SpinLock<T> {
    type G = SpinLockMutexGuard<'a, T>;

    fn new(value: T) -> Self where Self: Sized {
        SpinLock {
            lock: AtomicBool::new(false),
            owner: AtomicUsize::new(usize::MAX),
            data: UnsafeCell::new(value),
        }
    }

    fn lock(&self) -> LockResult<Self::G> {
        loop {
            match self.try_lock() {
                Some(guard) => return guard,
                None => continue,
            }
        }
    }

    fn try_lock(&self) -> TryLockResult<Self::G> {
        let me = aarch64::affinity();

        let ordering = if is_mmu_ready() {
            Ordering::SeqCst
        } else if me == 0 {
            Ordering::Relaxed
        } else {
            return Err(TryLockError::InvalidState);
        };

        //TODO: Review
        if !self.lock.compare_and_swap(false, true, ordering) {
            Ok(SpinLockMutexGuard(self))
        } else {
            Err(TryLockError::WouldBlock)
        }
    }

    fn unlock(&self) {
        let ordering = if is_mmu_ready() {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        self.owner.store(usize::MAX, ordering);
        self.lock.store(false, ordering);
    }

    fn is_poisoned(&self) -> bool {
        todo!()
    }

    fn clear_poison(&self) {
        todo!()
    }
}


impl<'a, T> !Send for SpinLockMutexGuard<'a, T> {}
unsafe impl<'a, T: Sync> Sync for SpinLockMutexGuard<'a, T> {}

