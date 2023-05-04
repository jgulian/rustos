use core::arch::asm;
use core::fmt;
use core::fmt::Write;
use core::time::Duration;

use shim::{ioerr, newioerr};

use crate::*;

macro_rules! err_or {
    ($ecode:expr, $rtn:expr) => {{
        let e = OsError::from($ecode);
        if let OsError::Ok = e {
            Ok($rtn)
        } else {
            Err(e)
        }
    }};
}

macro_rules! syscall_args {
    ($a:expr) => (
        asm!("mov x0, {}", in(reg) $a);
    );
    ($a:expr, $b:expr) => (
        syscall_args!($a);
        asm!("mov x1, {}", in(reg) $b);
    );
    ($a:expr, $b:expr, $c:expr) => (
        syscall_args!($a, $b);
        asm!("mov x2, {}", in(reg) $c);
    );
    ($a:expr, $b:expr, $c:expr, $d:expr) => (
        syscall_args!($a, $b, $c);
        asm!("mov x3, {}", in(reg) $d);
    );
    ($a:expr, $b:expr, $c:expr, $d:expr, $e:expr) => (
        syscall_args!($a, $b, $c, $d);
        asm!("mov x4, {}", in(reg) $e);
    );
}

macro_rules! syscall {
    ($syscall_number:expr) => {
        asm!("svc {}", const { $syscall_number as u16 })
    };
}

macro_rules! syscall_receive0 {
    () => {
        syscall_receive3!().map(|(_, _, _)| ())
    };
}

macro_rules! syscall_receive1 {
    () => {
        syscall_receive3!().map(|(x, _, _)| x)
    };
}

macro_rules! syscall_receive2 {
    () => {
        syscall_receive3!().map(|(x, y, _)| (x, y))
    };
}

macro_rules! syscall_receive3 {
    () => {{
        let x: u64;
        let y: u64;
        let z: u64;
        let e: u64;
        asm!(
            "mov {}, x0",
            "mov {}, x1",
            "mov {}, x2",
            "mov {}, x7",
            out(reg) x,
            out(reg) y,
            out(reg) z,
            out(reg) e
        );
        err_or!(e, (x, y, z))
    }};
}

pub fn sleep(span: Duration) -> OsResult<Duration> {
    if span.as_millis() > u64::MAX as u128 {
        panic!("too big!");
    }

    let elapsed_ms = unsafe {
        syscall_args!(span.as_millis() as u64);
        syscall!(Syscall::Sleep);
        syscall_receive1!()?
    };

    Ok(Duration::from_millis(elapsed_ms))
}

pub fn time() -> OsResult<Duration> {
    let returned = unsafe {
        syscall!(Syscall::Time);
        syscall_receive2!()?
    };

    Ok(Duration::from_secs(returned.0) + Duration::from_nanos(returned.1))
}

pub fn exit() -> OsResult<()> {
    unsafe {
        syscall!(Syscall::Exit);
        syscall_receive0!()
    }
}

pub fn open(file: &str) -> OsResult<u64> {
    unsafe {
        let slice = file.as_bytes();
        syscall_args!((slice.as_ptr()) as u64, slice.len() as u64);
        syscall!(Syscall::Open);
        syscall_receive1!()
    }
}

//TODO: make the semantics match io::Read
pub fn read(file: u64, bytes: &mut [u8]) -> OsResult<usize> {
    unsafe {
        syscall_args!(file, (bytes.as_ptr()) as u64, bytes.len() as u64);
        syscall!(Syscall::Read);
        syscall_receive1!().map(|size| size as usize)
    }
}

pub fn write(file: u64, bytes: &[u8]) -> OsResult<usize> {
    unsafe {
        syscall_args!(file, (bytes.as_ptr()) as u64, bytes.len() as u64);
        syscall!(Syscall::Write);
        syscall_receive1!().map(|size| size as usize)
    }
}

pub fn pipe() -> OsResult<(u64, u64)> {
    unsafe {
        syscall!(Syscall::Pipe);
        syscall_receive2!()
    }
}

pub fn getpid() -> OsResult<u64> {
    unsafe {
        syscall!(Syscall::GetPid);
        syscall_receive1!()
    }
}

pub fn sbrk() -> OsResult<(usize, usize)> {
    unsafe {
        syscall!(Syscall::Sbrk);
        let result = syscall_receive2!()?;
        Ok((result.0 as usize, result.1 as usize))
    }
}

pub fn fork() -> OsResult<Option<u64>> {
    let (child_id, is_child) = unsafe {
        syscall!(Syscall::Fork);
        syscall_receive2!()?
    };

    match is_child {
        0 => Ok(Some(child_id)),
        _ => Ok(None),
    }
}

pub fn duplicate(file: u64, new: u64) -> OsResult<u64> {
    unsafe {
        syscall_args!(file, new);
        syscall!(Syscall::Duplicate);
        syscall_receive1!()
    }
}

//TODO: this should not return on success; codify that
pub fn execute(arguments: &[u8], environment: &[u8]) -> OsResult<u64> {
    unsafe {
        syscall_args!(
            arguments.as_ptr() as u64,
            arguments.len() as u64,
            environment.as_ptr() as u64,
            environment.len() as u64
        );
        syscall!(Syscall::Execute);
        syscall_receive1!()
    }
}

pub fn wait(process: u64, timeout: Option<u64>) -> OsResult<Option<u64>> {
    let (pid, timed_out) = unsafe {
        syscall_args!(process, timeout.is_some() as u64, timeout.unwrap_or(0));
        syscall!(Syscall::Wait);
        syscall_receive2!()
    }?;
    if timed_out == 1 {
        Ok(None)
    } else {
        Ok(Some(pid))
    }
}

#[inline(never)]
pub(super) fn clone(start_address: usize, data: usize) -> OsResult<u64> {
    unsafe {
        syscall_args!(start_address, data);
        syscall!(Syscall::Clone);
        syscall_receive1!()
    }
}

pub fn switch_scheduler(policy: usize) -> OsResult<u64> {
    unsafe {
        syscall_args!(policy as u64);
        syscall!(Syscall::SwitchScheduler);
        syscall_receive1!()
    }
}
