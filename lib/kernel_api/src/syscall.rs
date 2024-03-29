use core::arch::asm;
use core::fmt;
use core::fmt::Write;
use core::time::Duration;

use shim::ioerr;

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
        _ => Ok(None)
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
        write(1, s.as_bytes()).expect("unable to write data");
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


// TODO: move to jlib
pub struct File(u64);
//TODO: Drop to close syscall

impl File {
    pub fn new(inner: u64) -> Self {
        File(inner)
    }
}

impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match read(self.0, buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => ioerr!(Interrupted),
        }
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match write(self.0, buf) {
            Ok(_) => Ok(buf.len()),
            Err(_) => ioerr!(Interrupted),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        ioerr!(Unsupported)
    }
}