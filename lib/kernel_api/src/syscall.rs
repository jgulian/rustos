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



unsafe fn syscall(call: Syscall, a: u64, b: u64, c: u64) -> OsResult<(u64, u64, u64)> {
    //FIXME: simplify this... it can be done (and probably should) in an inline macro
    let x: u64;
    let y: u64;
    let z: u64;
    let e: u64;
    asm!(
    "mov x0, {a}",
    "mov x1, {b}",
    "mov x2, {c}",
    "svc 0",
    "mov {x}, x0",
    "mov {y}, x1",
    "mov {z}, x2",
    "mov {e}, x7",
    a = in(reg) a,
    b = in(reg) b,
    c = in(reg) c,
    //t = in(reg) (call as u16),
    x = out(reg) x,
    y = out(reg) y,
    z = out(reg) z,
    e = out(reg) e,
    );

    err_or!(e, (x, y, z))
}

pub fn sleep(span: Duration) -> OsResult<Duration> {
    if span.as_millis() > core::u64::MAX as u128 {
        panic!("too big!");
    }

    let ms = span.as_millis() as u64;
    let elapsed_ms: u64;

    unsafe {
        elapsed_ms = syscall(Syscall::Sleep, ms, 0, 0)?.0
    }

    Ok(Duration::from_millis(elapsed_ms))
}

pub fn time() -> OsResult<Duration> {
    let returned = unsafe {
        syscall(Syscall::Time, 0, 0, 0)?
    };

    Ok(Duration::from_secs(returned.0) + Duration::from_nanos(returned.1))
}

pub fn exit() -> OsResult<()> {
    unsafe {
        syscall(Syscall::Exit, 0, 0, 0)?;
    }

    Ok(())
}

pub fn write(b: u8) -> OsResult<()> {
    unsafe {
        syscall(Syscall::Write, b as u64, 0, 0)?;
    }

    Ok(())
}

pub fn write_str(msg: &str) {
    for c in msg.bytes() {
        while write(c).is_err() {}
    }
}

pub fn getpid() -> OsResult<u64> {
    let pid = unsafe {
        syscall(Syscall::GetPid, 0, 0, 0)?.0
    };

    Ok(pid)
}

pub fn sbrk() -> OsResult<usize> {
    let ptr = unsafe {
        syscall(Syscall::Sbrk, 0, 0, 0)?.0 as usize
    };

    Ok(ptr)
}

struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
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