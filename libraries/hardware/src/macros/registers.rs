macro_rules! define_register_mask {
    ($bit:literal) => {
        (0b1 << $bit)
    }
    ($bit_beg:literal..$bit_end:literal) => {
        ((1 << ($bit_end + 1)) - (1 << $bit_beg))
    }
}

pub(crate) use define_register_mask;

macro_rules! define_register_bits_read {
    (Inner, $base:literal + $offset:literal, $size:ty, $name:ident,
    $bits:literal$(..$bits_end:literal)?) => {{
        use core::ptr::read_volatile;
        let data = unsafe {
            read_volatile(($base + $offset) as *const $size)
        };

        (data & define_register_mask!($bits$(..$bits_end)?)) >> $bits
    }}
    ($base:literal + $offset:literal, $size:ty/, $name:ident,
    $bits:literal$(..$bits_end:literal)?) => {
        pub fn read(&mut self) -> $size {
            define_register_bits_read!(
                Inner, $base + $offset, $size, $name, $bits$(..$bits_end)?
            )
        }
    }
    ($base:literal + $offset:literal, $size:ty/$bits_type:ty, $name:ident,
    $bits:literal$(..$bits_end:literal)?) => {
        pub fn read(&mut self) -> $bits_type {
            define_register_bits_read!(
                Inner, $base + $offset, $size, $name, $bits$(..$bits_end)?
            ) as $bits_type
        }
    }
}

pub(crate) use define_register_bits_read;

macro_rules! define_register_bits_write {
    (Inner, $base:literal + $offset:literal, $size:ty, $name:ident,
    $bits:literal$(..$bits_end:literal)?) => {
        use core::ptr::write_volatile;
        let mask = define_register_mask!($bits$(..$bits_end)?);
        self.0 = (self.0 & !mask) | ((data << $bits) & mask);
        unsafe {write_volatile(($base + $offset) as *const $size, self.0);}
    }
    ($base:literal + $offset:literal, $size:ty/, $name:ident,
    $bits:literal$(..$bits_end:literal)?) => {
        pub fn write(&mut self, data: $size) {
            define_register_bits_write!(
                Inner, $base + $offset, $size, $name, $bits$(..$bits_end)?
            );
        }
    }
    ($base:literal + $offset:literal, $size:ty/$bits_type:ty, $name:ident,
    $bits:literal$(..$bits_end:literal)?) => {
        pub fn write(&mut self, bits: $bits_type) {
            let data = bits as $size;
            define_register_bits_write!(
                Inner, $base + $offset, $size, $name, $bits$(..$bits_end)?
            );
        }
    }
}

pub(crate) use define_register_bits_write;

macro_rules! define_register_bits_impl {
    (Read, $base:literal + $offset:literal, $size:ty, $name:ident,
    $bits:literal $(..$bits_end:literal)? $(, $bits_type:ty)?) => {
        define_register_bits_read!($base + $offset, $size/$($bits_type)?, $name, $bits$(..$bits_end)?)
    }
    (Write, $base:literal + $offset:literal, $size:ty, $name:ident,
    $bits:literal $(..$bits_end:literal)? $(, $bits_type:ty)?) => {
        define_register_bits_write!($base + $offset, $size/$($bits_type)?, $name, $bits$(..$bits_end)?)
    }
    (ReadWrite, $base:literal + $offset:literal, $size:ty, $name:ident,
    $bits:literal $(..$bits_end:literal)? $(, $bits_type:ty)?) => {
        define_register_bits_read!($base + $offset, $size/$($bits_type)?, $name, $bits$(..$bits_end)?)
        define_register_bits_write!($base + $offset, $size/$($bits_type)?, $name, $bits$(..$bits_end)?)
    }
}

pub(crate) use define_register_bits_impl;

macro_rules! define_registers {
    ($module:ident, $base:literal, [
        $(
            ($register:ident, $size:ty, $offset:literal): [
                $(($name:ident, $bits:literal$(..$bits_end:literal)?, $RW:ident $(, $bits_type:ty)?),)*
            ],
        )+
    ]) => {
        mod $module {
            $(
            pub mod $register {
                use crate::macros::registers::*;
                
                $(
                    pub struct $name($size);

                    impl $name {
                        pub fn new() -> Self {
                            Self ($size::default())
                        }
                        
                        define_register_bits_impl!(
                            $RW, $base + $offset, $size, $name, $bits$(..$bits_end)? $(, $bits_type)?
                        )
                    }
                )*
            }
            )*
        }
    }
}

pub(crate) use define_registers;