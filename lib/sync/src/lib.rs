#![feature(negative_impls)]
#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;

use core::ops::DerefMut;
use core::ops::FnOnce;
use core::result::Result;

#[derive(Debug)]
pub enum LockError {
    Poisoned,
    InvalidState,
    WouldBlock,
}

pub type LockResult<T> = Result<T, LockError>;

pub trait Mutex<T: Send>: Send + Sync {
    fn new(value: T) -> Self
    where
        Self: Sized;
    fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R>;
    fn try_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> LockResult<R>;
    fn is_poisoned(&self) -> bool;
    fn clear_poison(&self);
}
