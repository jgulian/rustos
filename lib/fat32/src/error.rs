use shim::io;

pub(crate) enum VirtualFatError {
    Io(io::Error),
    InvalidClusterForNext,
    FilesystemOutOfMemory,
    FailedToLockFatMutex,
    InvalidFatForSizing,
}

impl From<io::Error> for VirtualFatError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub(crate) type VirtualFatResult<T> = Result<T, VirtualFatError>;