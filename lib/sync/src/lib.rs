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
    InvalidState,
    WouldBlock,
}

pub type LockResult<G> = Result<G, PoisonError<G>>;

pub type TryLockResult<G> = Result<G, TryLockError<G>>;

pub trait Mutex<T: Send>: Send + Sync {
    fn new(value: T) -> Self where Self: Sized;
    fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R>;
    fn try_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> TryLockResult<R>;
    fn is_poisoned(&self) -> bool;
    fn clear_poison(&self);
}