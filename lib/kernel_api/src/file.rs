use core::fmt;
use core::fmt::Write;
use shim::{io, ioerr, newioerr};
use crate::syscall::{read, write};

struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(1, s.as_bytes()).expect("unable to write data");
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::file::vprint(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
 () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::file::vprint(format_args!($($arg)*));
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
        read(self.0, buf).map_err(|_| newioerr!(Interrupted))
    }
}

impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(self.0, buf).map_err(|_| newioerr!(Interrupted))
    }

    fn flush(&mut self) -> io::Result<()> {
        ioerr!(Unsupported)
    }
}