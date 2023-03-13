#![feature(negative_impls)]

#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

use core::result::Result;
use core::ops::{Deref, DerefMut};

pub struct PoisonError<G> {
    guard: G,
}

pub enum TryLockError<G> {
    Poisoned(PoisonError<G>),
    WouldBlock,
}

pub type LockResult<G> = Result<G, PoisonError<G>>;

pub type TryLockResult<G> = Result<G, TryLockError<G>>;

pub trait Mutex<T: Sized> {
    type G: MutexGuard<T>;

    fn new(value: T) -> Self where Self: Sized;
    fn lock(&self) -> LockResult<Self::G>;
    fn try_lock(&self) -> TryLockResult<Self::G>;
    fn unlock(&self);
    fn is_poisoned(&self) -> bool;
    fn clear_poison(&self);
}

pub trait MutexGuard<T>: Deref<Target=T> + DerefMut<Target=T> {}