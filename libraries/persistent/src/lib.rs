#![cfg_attr(feature = "no_std", no_std)]

extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "no_std")] {
        mod no_std;
        pub use self::no_std::*;
    } else {
        mod std;
        pub use self::std::*;
    }
}

pub mod filesystem;

