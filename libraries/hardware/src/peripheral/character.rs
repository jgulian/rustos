use shim::io;

pub trait CharacterDevice {
    fn read_byte(&mut self) -> io::Result<u8>;
    fn write_byte(&mut self, byte: u8) -> io::Result<()>;
    fn flush(&mut self) -> io::Result<()>;
}

impl io::Read for dyn CharacterDevice {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        buf.iter_mut().fold(Ok(0), |amount_read_wrapped, byte| {
            let amount_read = amount_read_wrapped?;
            *byte = self.read_byte()?;
            Ok(amount_read + 1)
        })
    }
}

impl io::Write for dyn CharacterDevice {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        buf.into().fold(Ok(0), |amount_written_wrapped, byte| {
            let amount_written = amount_written_wrapped?;
            self.write_byte(byte)?;
            Ok(amount_written + 1)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        CharacterDevice::flush(self)
    }
}

