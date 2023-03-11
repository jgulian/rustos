#![feature(negative_impls)]

#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

use core::marker::Send;
use core::ops::Drop;

pub struct MutexGuard<'a, T: 'a> {
    mutex: &'a mut dyn Mutex<T>,
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock()
    }
}

impl<T> !Send for MutexGuard<'_, T> {}

pub struct PoisonError<G> {
    guard: G,
}

pub enum TryLockError<G> {
    Poisoned(PoisonError<G>),
    WouldBlock,
}

pub type LockResult<G> = core::result::Result<G, PoisonError<G>>;

pub type TryLockResult<G> = core::result::Result<G, TryLockError<G>>;

pub trait Mutex<T> {
    fn lock(&self) -> LockResult<MutexGuard<'_, T>>;
    fn try_lock(&self) -> TryLockResult<MutexGuard<'_, T>>;
    fn unlock(&self);
    fn is_poisoned(&self) -> bool;
    fn clear_poison(&self);
}