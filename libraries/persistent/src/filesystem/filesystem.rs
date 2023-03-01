use shim::io;
use crate::filesystem::path::Path;

pub enum Error {
    Unsupported,
    InvalidPath,
    EntryNotFound,
}

pub type Result<T> = core::result::Result<T, Error>;

pub trait File: io::Read + io::Write + io::Seek {
    fn duplicate(&mut self) -> Result<Self>;
}

pub trait Directory {

}

pub trait Filesystem {

}