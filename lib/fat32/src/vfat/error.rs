use shim::io;

#[derive(Debug)]
pub enum Error {
    Mbr(mbr::Error),
    Io(io::Error),
    BadSignature,
    NotFound,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Io(error)
    }
}
