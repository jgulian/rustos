use shim::io;

#[derive(Debug)]
pub enum FilesystemError {
    /// There was an I/O error while reading the device.
    Io(io::Error),
    /// Invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The device had a bad signature
    BadSignature,
}

impl From<io::Error> for FilesystemError {
    fn from(value: io::Error) -> Self {
        FilesystemError::Io(value)
    }
}
