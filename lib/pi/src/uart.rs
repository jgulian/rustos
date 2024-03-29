use core::fmt;
use core::time::Duration;

use filesystem::CharDevice;
use shim::io;
use shim::ioerr;
use volatile::{Reserved, Volatile};
use volatile::prelude::*;

use crate::common::IO_BASE;
use crate::gpio::{Function, Gpio};
use crate::timer;

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    MU_IO: Volatile<u8>,
    __r0: [Reserved<u8>; 3],
    MU_IER: Volatile<u8>,
    __r1: [Reserved<u8>; 3],
    MU_IIR: Volatile<u8>,
    __r2: [Reserved<u8>; 3],
    MU_LCR: Volatile<u8>,
    __r3: [Reserved<u8>; 3],
    MU_MCR: Volatile<u8>,
    __r4: [Reserved<u8>; 3],
    MU_LSR: Volatile<u8>,
    __r5: [Reserved<u8>; 3],
    MU_MSR: Volatile<u8>,
    __r6: [Reserved<u8>; 3],
    MU_SCRATCH: Volatile<u8>,
    __r7: [Reserved<u8>; 3],
    MU_CNTL: Volatile<u8>,
    __r8: [Reserved<u8>; 3],
    MU_STAT: Volatile<u32>,
    MU_BAUD: Volatile<u16>,
    __r9: [Reserved<u8>; 2],
}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<Duration>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        registers.MU_CNTL.write(0);
        registers.MU_LCR.write(0b11);
        registers.MU_BAUD.write(270);

        Gpio::new(14).into_alt(Function::Alt5);
        Gpio::new(15).into_alt(Function::Alt5);

        registers.MU_CNTL.write(0b11);

        MiniUart {
            registers,
            timeout: None,
        }
    }

    /// Set the read timeout to `t` duration.
    pub fn set_read_timeout(&mut self, t: Duration) {
        self.timeout = Some(t);
    }

    pub fn can_write(&mut self) -> bool {
        (self.registers.MU_LSR.read() & (LsrStatus::TxAvailable as u8)) != 0
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        self.registers.MU_IO.write(byte)
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        (self.registers.MU_LSR.read() & (LsrStatus::DataReady as u8)) > 0
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        match self.timeout {
            Some(timeout) => {
                let end = timer::current_time() + timeout;

                while !self.has_byte() && timer::current_time() < end {}

                if timer::current_time() < end {
                    Ok(())
                } else {
                    Err(())
                }
            }
            None => {
                while !self.has_byte() {}
                Ok(())
            }
        }
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        while !self.has_byte() {}
        self.registers.MU_IO.read()
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.as_bytes() {
            if *c == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(*c);
        }
        Ok(())
    }
}

impl io::Read for MiniUart {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.wait_for_byte() {
            Ok(_) => {
                for i in 0..buf.len() {
                    if !self.has_byte() {
                        return Ok(i);
                    }
                    buf[i] = self.read_byte();
                }
                Ok(buf.len())
            }
            Err(_) => ioerr!(TimedOut, "read timed out"),
        }
    }
}

impl io::Write for MiniUart {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for byte in buf.iter() {
            self.write_byte(*byte);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
