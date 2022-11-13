use core::arch::asm;
use core::fmt;
use core::fmt::Write;
use core::time::Duration;

pub fn time() -> Duration {
    let mut ecode: u64;
    let mut elapsed_s: u64;
    let mut elapsed_ns: u64;

    unsafe {
        asm!(
        "svc 1",
        "mov {x}, x0",
        "mov {y}, x1",
        "mov {e}, x7",
        x = out(reg) elapsed_s,
        y = out(reg) elapsed_ns,
        e = out(reg) ecode,
        );
    }

    Duration::from_secs(elapsed_s) + Duration::from_nanos(elapsed_ns)
}

pub fn exit() -> ! {
    unsafe {
        asm!("svc 2");
    }

    loop {}
}

pub fn write(file: u64, ptr: usize, len: usize) {
    let mut ecode: u64;

    unsafe {
        asm!(
        "mov x0, {a}",
        "mov x1, {b}",
        "mov x2, {c}",
        "svc 5",
        "mov {e}, x7",
        a = in(reg) (file),
        b = in(reg) (ptr as u64),
        c = in(reg) (len as u64),
        e = out(reg) ecode,
        );
    }
}

pub fn getpid() -> u64 {
    let mut ecode: u64;
    let mut pid: u64;

    unsafe {
        asm!(
        "svc 6",
        "mov {x}, x0",
        "mov {e}, x7",
        x = out(reg) pid,
        e = out(reg) ecode,
        );
    }

    pid
}


pub fn sbrk() -> (usize, usize) {
    let mut ecode: u64;
    let mut ptr: u64;
    let mut len: u64;

    unsafe {
        asm!(
        "svc 7",
        "mov {x}, x0",
        "mov {y}, x1",
        "mov {e}, x7",
        x = out(reg) ptr,
        y = out(reg) len,
        e = out(reg) ecode,
        );
    }

    if ecode == 1 {
        (ptr as usize, len as usize)
    } else {
        (0, 0)
    }
}

struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(0, s.as_ptr() as usize, s.len());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel_api::vprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
 () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::kernel_api::vprint(format_args!($($arg)*));
        $crate::print!("\n");
    })
}

pub fn vprint(args: fmt::Arguments) {
    let mut c = Console;
    c.write_fmt(args).unwrap();
}