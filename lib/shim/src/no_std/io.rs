use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{cmp, result};
use core::fmt::{Debug, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    FilesystemQuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
}

pub struct Error {
    kind: ErrorKind,
    repr: &'static str,
}

impl Error {
    pub fn new(kind: ErrorKind, repr: &'static str) -> Error {
        Error {
            kind,
            repr,
        }
    }

    pub fn from(kind: ErrorKind) -> Error {
        Error {
            kind,
            repr: "",
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:?}: {}", self.kind, self.repr))
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum SeekFrom {
    Start(u64),
    End(i64),
    Current(i64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64>;

    fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }
    fn stream_len(&mut self) -> Result<u64> {
        let location = self.stream_position()?;
        let length = self.seek(SeekFrom::End(0))?;
        self.seek(SeekFrom::Start(location))?;
        Ok(length)
    }
    fn stream_position(&mut self) -> Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut i = 0;
        while i < buf.len() {
            match self.read(&mut buf[i..]) {
                Ok(0) => {
                    Err(Error::from(ErrorKind::UnexpectedEof))
                }
                Ok(n) => {
                    i += n;
                    Ok(())
                }
                Err(error) => {
                    if error.kind() == ErrorKind::Interrupted {
                        Ok(())
                    } else {
                        Err(error)
                    }
                }
            }?;
        }

        Ok(())
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let starting_size = buf.len();
        while {
            let mut buffer = [0u8; 256];
            let amount_read = self.read(&mut buffer)?;
            buf.extend_from_slice(&buffer);
            amount_read != 0
        } {}

        Ok(buf.len() - starting_size)
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    fn flush(&mut self) -> Result<()>;

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut i = 0;
        while i < buf.len() {
            match self.write(&buf[i..]) {
                Ok(0) => {
                    return Err(Error::from(ErrorKind::WriteZero));
                }
                Ok(n) => {
                    i += n;
                }
                Err(e) => {
                    if e.kind() != ErrorKind::Interrupted {
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    data: T,
    position: u64,
}

impl<T> Cursor<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            position: 0,
        }
    }
}

impl<T> Seek for Cursor<T> where T: AsRef<[u8]> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(n) => Some(n as i64),
            SeekFrom::End(n) => (self.data.as_ref().len() as i64).checked_sub(n),
            SeekFrom::Current(n) => (self.position as i64).checked_add(n),
        }.ok_or(Error::from(ErrorKind::InvalidInput))?;

        if new_position < 0 || self.data.as_ref().len() <= new_position as usize {
            Err(Error::from(ErrorKind::InvalidInput))
        } else {
            Ok(new_position as u64)
        }
    }
}

impl Read for &mut [u8] {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let length = cmp::min(buf.len(), self.len());
        for i in 0..length {
            buf[i] = self[i];
        }

        Ok(length)
    }
}

fn read_with_buf(buf: &mut [u8], position: &mut u64, target: &mut [u8]) -> Result<usize> {
    let read = (&mut buf[(*position as usize)..]).read(target)?;
    *position += read as u64;
    Ok(read)
}

impl Read for Cursor<Box<[u8]>> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        read_with_buf((*self.data).as_mut(), &mut self.position, buf)
    }
}

impl Read for Cursor<Vec<u8>> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        read_with_buf(self.data.as_mut_slice(), &mut self.position, buf)
    }
}

impl Read for Cursor<&mut [u8]> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        read_with_buf(self.data, &mut self.position, buf)
    }
}

// FIXME: use copy_from_slice

impl Write for &mut [u8] {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let length = cmp::min(buf.len(), self.len());
        for i in 0..length {
            self[i] = buf[i];
        }
        Ok(length)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

fn write_with_buf(buf: &mut [u8], position: &mut u64, target: &[u8]) -> Result<usize> {
    let written = (&mut buf[(*position as usize)..]).write(target)?;
    *position += written as u64;
    Ok(written)
}

impl Write for Cursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        write_with_buf(self.data.as_mut_slice(), &mut self.position, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Write for Cursor<Box<[u8]>> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        write_with_buf((*self.data).as_mut(), &mut self.position, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Write for Cursor<&mut [u8]> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        write_with_buf(self.data, &mut self.position, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}