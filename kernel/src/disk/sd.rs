use core::time::Duration;
use filesystem::device::BlockDevice;

use shim::io;

extern "C" {
    /// A global representing the last SD controller error that occured.
    static sd_err: i64;

    /// Initializes the SD card controller.
    ///
    /// Returns 0 if initialization is successful. If initialization fails,
    /// returns -1 if a timeout occured, or -2 if an error sending commands to
    /// the SD controller occured.
    fn sd_init() -> i32;

    /// Reads sector `n` (512 bytes) from the SD card and writes it to `buffer`.
    /// It is undefined behavior if `buffer` does not point to at least 512
    /// bytes of memory. Also, the caller of this function should make sure that
    /// `buffer` is at least 4-byte aligned.
    ///
    /// On success, returns the number of bytes read: a positive number.
    ///
    /// On error, returns 0. The true error code is stored in the `sd_err`
    /// global. `sd_err` will be set to -1 if a timeout occured or -2 if an
    /// error sending commands to the SD controller occured. Other error codes
    /// are also possible but defined only as being less than zero.
    fn sd_readsector(n: i32, buffer: *mut u8) -> i32;
}

// FIXME: Define a `#[no_mangle]` `wait_micros` function for use by `libsd`.
// The `wait_micros` C signature is: `void wait_micros(unsigned int);`
#[no_mangle]
fn wait_micros(micros: u32) {
    pi::timer::spin_sleep(Duration::from_micros(micros as u64));
}

/// A handle to an SD card controller.
#[derive(Debug)]
pub struct Sd;

impl Sd {
    /// Initializes the SD card controller and returns a handle to it.
    /// The caller should assure that the method is invoked only once during the
    /// kernel2 initialization. We can enforce the requirement in safe Rust code
    /// with atomic memory access, but we can't use it yet since we haven't
    /// written the memory management unit (MMU).
    pub unsafe fn new() -> Result<Sd, io::Error> {
        let result = sd_init();
        match result {
            0 => Ok(Sd),
            _ => Err(io::Error::from(io::ErrorKind::Other)),
        }
    }
}

impl BlockDevice for Sd {
    fn block_size(&self) -> usize {
        512
    }

    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()> {
        if data.len() < 512 || block > (1 << (31 - 1)) {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        // Check alignment
        let did_err = unsafe {
            let ptr = data.as_mut_ptr();
            if (ptr as usize) % 4 != 0 {
                return Err(io::Error::from(io::ErrorKind::InvalidInput));
            }

            sd_readsector(block as i32, ptr) == 0
        };

        if did_err {
            match unsafe { sd_err } {
                -1 => Err(io::Error::from(io::ErrorKind::TimedOut)),
                _ => Err(io::Error::from(io::ErrorKind::Other))
            }
        } else {
            Ok(())
        }
    }

    fn write_block(&mut self, _block: u64, _data: &[u8]) -> io::Result<()> {
        unimplemented!("SD card and file system are read only")
    }
}
