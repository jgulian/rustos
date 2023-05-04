#![no_std]
#![feature(asm_const)]

extern crate alloc;

use shim::io;

#[cfg(feature = "user-space")]
pub mod file;
#[cfg(feature = "user-space")]
pub mod syscall;
#[cfg(feature = "user-space")]
pub mod thread;

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
    UnknownResourceId = 80,
    WouldBlock = 90,

    IoError = 101,
    IoErrorEof = 102,
    IoErrorInvalidData = 103,
    IoErrorInvalidInput = 104,
    IoErrorTimedOut = 105,

    InvalidSocket = 200,
    IllegalSocketOperation = 201,

    SchedulerError = 210,
}

impl From<u64> for OsError {
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
            210 => OsError::SchedulerError,

            _ => OsError::Unknown,
        }
    }
}

impl From<io::Error> for OsError {
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

#[derive(Debug)]
pub enum Syscall {
    Open = 0,
    Close = 1,
    Read = 2,
    Write = 3,
    Pipe = 4,
    Duplicate = 5,
    Seek = 6,

    Fork = 10,
    Execute = 11,
    Exit = 12,
    Wait = 13,
    GetPid = 14,
    Clone = 15,

    Sbrk = 20,

    Sleep = 30,
    Time = 31,

    SwitchScheduler = 32,

    Unknown = 256,
}

impl From<u16> for Syscall {
    fn from(value: u16) -> Self {
        match value {
            0 => Syscall::Open,
            1 => Syscall::Close,
            2 => Syscall::Read,
            3 => Syscall::Write,
            4 => Syscall::Pipe,
            5 => Syscall::Duplicate,
            6 => Syscall::Seek,

            10 => Syscall::Fork,
            11 => Syscall::Execute,
            12 => Syscall::Exit,
            13 => Syscall::Wait,
            14 => Syscall::GetPid,
            15 => Syscall::Clone,

            20 => Syscall::Sbrk,

            30 => Syscall::Sleep,
            31 => Syscall::Time,

            32 => Syscall::SwitchScheduler,

            _ => Syscall::Unknown,
        }
    }
}
