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

pub fn write(b: u8) {
    let mut ecode: u64;

    unsafe {
        asm!(
        "mov x0, {a}",
        "svc 5",
        "mov {e}, x7",
        a = in(reg) (b as u64),
        e = out(reg) ecode,
        );
    }
}

pub fn write_str(msg: &str) {
    let pad = [0; 64];
    for c in msg.bytes() {
        write(c);
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

struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write_str(s);
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