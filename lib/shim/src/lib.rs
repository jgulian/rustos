#![cfg_attr(feature = "no_std", no_std)]
#![feature(str_internals)]
#![feature(never_type)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "no_std")]
mod no_std;

#[cfg(feature = "no_std")]
pub use self::no_std::io;

#[cfg(not(feature = "no_std"))]
mod std;
#[cfg(not(feature = "no_std"))]
pub use self::std::*;

#[cfg(feature = "no_std")]
compile_error!("This macro only accepts `foo` or `bar`");

#[macro_use]
pub mod macros;

#[cfg(test)]
mod tests;