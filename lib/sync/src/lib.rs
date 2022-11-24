#![no_std]

use core::marker::Sized;
use core::ops::{Deref, DerefMut, Drop, FnOnce};

pub trait Mutex: Sized {
    fn new() -> Self;
    fn lock(&mut self);
    fn unlock(&mut self);
}

pub struct Latch<M: Mutex, T: Sized> {
    mutex: M,
    data: T,
}

impl<'a, M: Mutex, T: Sized> Latch<M, T> {
    pub fn new(data: T) -> Self {
        Latch {
            mutex: M::new(),
            data,
        }
    }

    pub fn guard(&'a mut self) -> LatchGuard<'a, M, T> {
        LatchGuard(self)
    }

    pub fn critical<F: FnOnce(&T) -> R, R>(&mut self, function: F) -> R {
        let lock_guard = self.guard();
        function(&*lock_guard)
    }

    pub fn critical_mut<F: FnOnce(&mut T) -> R, R>(&mut self, function: F) -> R {
        let mut lock_guard = self.guard();
        function(&mut *lock_guard)
    }
}

pub struct LatchGuard<'a, M: Mutex, T: Sized>(&'a mut Latch<M, T>);

impl<'a, M: Mutex, T: Sized> Deref for LatchGuard<'a, M, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}

impl<'a, M: Mutex, T: Sized> DerefMut for LatchGuard<'a, M, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.data
    }
}

impl<'a, M: Mutex, T: Sized> Drop for LatchGuard<'a, M, T> {
    fn drop(&mut self) {
        self.0.mutex.unlock();
    }
}