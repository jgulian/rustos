#![no_std]

use core::marker::Sized;
use core::ops::{Deref, DerefMut, Drop};

pub struct PoisonError<G>(G);

pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

pub trait Guard<'a, T: Sized + 'a>: Deref<Target=T> + DerefMut<Target=T> {}

pub trait Mutex<'a, T: Sized + 'a> {
    type Guard: Guard<'a, T>;

    fn new(data: T) -> Self;
    fn lock(&mut self) -> LockResult<Self::Guard>;
}