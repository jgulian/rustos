#![cfg_attr(feature = "no_std", no_std)]

pub use format_derive::Format;

use shim::io::{Cursor, Read, Seek, Result, Write};

pub trait Format: Sized {
    fn load_readable<T: Read>(stream: &mut T) -> Result<Self>;
    fn load_readable_seekable<T: Read + Seek>(stream: &mut T) -> Result<Self>;
    fn save_writable<T: Write>(&self, stream: &mut T) -> Result<()>;
    fn save_writable_seekable<T: Write + Seek>(&self, stream: &mut T) -> Result<()>;

    //TODO: add these to derive format
    fn load_slice(slice: &[u8]) -> Result<Self> {
        let mut vec = Vec::new();
        vec.extend_from_slice(slice);
        Self::load_readable_seekable(&mut Cursor::new(vec))
    }
    fn save_slice(&self, slice: &mut [u8]) -> Result<()> {
        self.save_writable(&mut Cursor::new(slice))
    }
}

//TODO: use custom errors as opposed to io errors
//TODO: allow users to specify a default endianness?

#[cfg(test)]
pub mod tests;