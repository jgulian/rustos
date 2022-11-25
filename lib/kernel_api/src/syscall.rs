use core::arch::asm;
use core::fmt;
use core::fmt::Write;
use core::time::Duration;

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
}

macro_rules! syscall {
    ($syscall_number:expr) => (
        asm!("svc {}", const { $syscall_number as u16 })
    );
}

macro_rules! syscall_receive0 {
    () => {{
        let e: u64;
        asm!("mov {}, x7", out(reg) e);
        err_or!(e, ())
    }};
}

macro_rules! syscall_receive1 {
    () => {{
        let _ = syscall_receive0!()?;
        let x: u64;
        asm!("mov {}, x0", out(reg) x);
        Ok(x) as Result<u64, OsError>
    }};
}

macro_rules! syscall_receive2 {
    () => {{
        let x = syscall_receive1!()?;
        let y: u64;
        asm!("mov {}, x1", out(reg) y);
        Ok((x, y)) as Result<(u64, u64), OsError>
    }};
}

macro_rules! syscall_receive3 {
    () => {{
        let p = syscall_receive2!()?;
        let z: u64;
        asm!("mov {}, x2", out(reg) z);
        Ok((p.0, p.1, z)) as Result<(u64, u64, u64), OsError>
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

pub fn read(file: u64, bytes: &mut [u8]) -> OsResult<()> {
    unsafe {
        syscall_args!(file, (bytes.as_ptr()) as u64, bytes.len() as u64);
        syscall!(Syscall::Read);
        syscall_receive0!()
    }
}

pub fn write(file: u64, bytes: &[u8]) -> OsResult<()> {
    unsafe {
        syscall_args!(file, (bytes.as_ptr()) as u64, bytes.len() as u64);
        syscall!(Syscall::Write);
        syscall_receive0!()
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

pub fn fork() -> OsResult<u64> {
    unsafe {
        syscall!(Syscall::Fork);
        syscall_receive1!()
    }
}

pub fn duplicate(file: u64) -> OsResult<u64> {
    unsafe {
        syscall_args!(file);
        syscall!(Syscall::Duplicate);
        syscall_receive1!()
    }
}

//TODO: this should not return on success; codify that
pub fn execute(arguments: &[u8], environment: &[u8]) -> OsResult<u64> {
    unsafe {
        syscall_args!(arguments.as_ptr() as u64, arguments.len() as u64,
            environment.as_ptr() as u64, environment.len() as u64);
        syscall!(Syscall::Execute);
        syscall_receive1!()
    }
}

pub fn wait(process: u64) -> OsResult<u64> {
    unsafe {
        syscall_args!(process);
        syscall!(Syscall::Wait);
        syscall_receive1!()
    }
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(0, s.as_bytes()).expect("unable to write data");
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::syscall::vprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
 () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::syscall::vprint(format_args!($($arg)*));
        $crate::print!("\n");
    })
}

pub fn vprint(args: fmt::Arguments) {
    let mut c = Console;
    c.write_fmt(args).unwrap();
}