use shim::io::{Read, Seek, Result, Write};

pub trait Format: Sized {
    fn load_readable<T: Read>(stream: &mut T) -> Result<Self>;
    fn load_readable_seekable<T: Read + Seek>(stream: &mut T) -> Result<Self>;
    fn save_writable<T: Write>(&self, stream: &mut T) -> Result<()>;
    fn save_writable_seekable<T: Write + Seek>(&self, stream: &mut T) -> Result<()>;
}

// #[cfg(test)]
pub mod tests;