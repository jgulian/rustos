use shim::io::{Read, Seek, Result, Write};

pub trait Formatted {
    fn load_readable<T: Read>(stream: &mut T) -> Result<Self>;
    fn load_readable_seekable<T: Read + Seek>(stream: &mut T) -> Result<Self>;
    fn save_writable<T: Write>(&self, stream: &mut T);
    fn save_writable_seekable<T: Write + Seek>(&self, stream: &mut T);
}

#[cfg(test)]
pub mod tests;