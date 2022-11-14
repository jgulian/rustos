use shim::io;

pub trait File: io::Read + io::Seek {}

impl<T> File for T where T: io::Read + io::Seek {}


pub(crate) trait FromBytes<const N: usize> {
    fn from_be_bytes(buf: [u8; N]) -> Self;
    fn from_le_bytes(buf: [u8; N]) -> Self;
    fn from_ne_bytes(buf: [u8; N]) -> Self;
}

impl FromBytes<2> for u16 {
    fn from_be_bytes(buf: [u8; 2]) -> Self {
        u16::from_be_bytes(buf)
    }

    fn from_le_bytes(buf: [u8; 2]) -> Self {
        u16::from_le_bytes(buf)
    }

    fn from_ne_bytes(buf: [u8; 2]) -> Self {
        u16::from_ne_bytes(buf)
    }
}

impl FromBytes<4> for u32 {
    fn from_be_bytes(buf: [u8; 4]) -> Self {
        u32::from_be_bytes(buf)
    }

    fn from_le_bytes(buf: [u8; 4]) -> Self {
        u32::from_le_bytes(buf)
    }

    fn from_ne_bytes(buf: [u8; 4]) -> Self {
        u32::from_ne_bytes(buf)
    }
}

impl FromBytes<8> for u64 {
    fn from_be_bytes(buf: [u8; 8]) -> Self {
        u64::from_be_bytes(buf)
    }

    fn from_le_bytes(buf: [u8; 8]) -> Self {
        u64::from_le_bytes(buf)
    }

    fn from_ne_bytes(buf: [u8; 8]) -> Self {
        u64::from_ne_bytes(buf)
    }
}

pub(crate) enum BitWidth {
    Bit64,
    Bit32,
    Unknown,
}

pub(crate) enum Endianness {
    Big,
    Little,
    Unknown,
}