use shim::io;

pub trait ByteDevice {
    fn read_byte(&mut self) -> io::Result<u8>;
    fn write_byte(&mut self, byte: u8) -> io::Result<()>;
}

pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn read_block(&mut self, block: u64, data: &mut [u8]) -> io::Result<()>;
    fn write_block(&mut self, block: u64, data: &[u8]) -> io::Result<()>;
}