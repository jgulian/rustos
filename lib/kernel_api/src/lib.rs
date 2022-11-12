#![feature(asm_const)]

#![no_std]

use shim::io;

#[cfg(feature = "user-space")]
pub mod syscall;

pub type OsResult<T> = Result<T, OsError>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OsError {
    Unknown = 0,
    Ok = 1,

    NoEntry = 10,
    NoMemory = 20,
    NoVmSpace = 30,
    NoAccess = 40,
    BadAddress = 50,
    FileExists = 60,
    InvalidArgument = 70,

    IoError = 101,
    IoErrorEof = 102,
    IoErrorInvalidData = 103,
    IoErrorInvalidInput = 104,
    IoErrorTimedOut = 105,

    InvalidSocket = 200,
    IllegalSocketOperation = 201,
}

impl core::convert::From<u64> for OsError {
    fn from(e: u64) -> Self {
        match e {
            1 => OsError::Ok,

            10 => OsError::NoEntry,
            20 => OsError::NoMemory,
            30 => OsError::NoVmSpace,
            40 => OsError::NoAccess,
            50 => OsError::BadAddress,
            60 => OsError::FileExists,
            70 => OsError::InvalidArgument,

            101 => OsError::IoError,
            102 => OsError::IoErrorEof,
            103 => OsError::IoErrorInvalidData,
            104 => OsError::IoErrorInvalidInput,

            200 => OsError::InvalidSocket,
            201 => OsError::IllegalSocketOperation,

            _ => OsError::Unknown,
        }
    }
}

impl core::convert::From<io::Error> for OsError {
    fn from(e: io::Error) -> Self {
        match e.kind() {
            io::ErrorKind::UnexpectedEof => OsError::IoErrorEof,
            io::ErrorKind::InvalidData => OsError::IoErrorInvalidData,
            io::ErrorKind::InvalidInput => OsError::IoErrorInvalidInput,
            io::ErrorKind::TimedOut => OsError::IoErrorTimedOut,
            io::ErrorKind::NotFound => OsError::NoEntry,
            _ => OsError::IoError,
        }
    }
}

pub enum Syscall {
    Sleep = 0,
    Time = 1,
    Exit = 2,
    Open = 3,
    Read = 4,
    Write = 5,
    GetPid = 6,
    Sbrk = 7,
    Unknown = 256,
}

impl From<u16> for Syscall {
    fn from(value: u16) -> Self {
        match value {
            0 => Syscall::Sleep,
            1 => Syscall::Time,
            2 => Syscall::Exit,
            3 => Syscall::Open,
            4 => Syscall::Read,
            5 => Syscall::Write,
            6 => Syscall::GetPid,
            7 => Syscall::Sbrk,
            _ => Syscall::Unknown,
        }
    }
}