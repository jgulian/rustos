//TODO: there is no readwrite, add this

macro_rules! mask {
    ($bit:literal) => {
        (0b1 << $bit)
    }
    ($bit_beg:literal..$bit_end:literal) => {
        ((1 << ($bit_end + 1)) - (1 << $bit_beg))
    }
}

pub(crate) use mask;

macro_rules! field_type {
    ($First:ident,) => { $First }
    ($First:ident, $Second:ident) => { $Second }
    ($First:ident,,) => { $First }
    ($First:ident, $Second:ident,) => { $Second }
    ($First:ident, $($Second:ident)?,$Third:ident) => { $Third }
}

pub(crate) use field_type;

macro_rules! default_value {
    ($RegisterType:ident,,) => {$RegisterType::default()}
    ($RegisterType:ident,$FieldType:ident,) => {$FieldType::default()}
    ($RegisterType:ident,$($FieldType:ident)?,$default:expr) => {$default}
}

pub(crate) use default_value;

macro_rules! define_registers {
    ($module:ident, $base:literal, [
        $(
            ($Register:ident, $RegisterType:ident, $offset:literal): [
                $(($field:ident, $bits:literal$(..$bits_end:literal)?, $ReadWrite:ident, {
                    $(FieldType: $FieldType:ident,)?
                    $(CustomType: $CustomType:ident {$($CTypeName:ident = $CTypeValue:literal,)+},)?
                    $(DefaultValue: $DefaultValue:literal,)?
                }),)*
            ],
        )*
    ]) => {
        mod $module {
            use core::default::Default;
            use crate::macros::registers::*;

            $(

            $(
                $(
                    #[repr($RegisterType)]
                    pub enum $CustomType {
                        $(
                            $CTypeName = $CTypeValue,
                        )+
                    }
                )?
            )*

            pub struct $Register {
                $(
                    pub $field: field_type!($RegisterType, $($FieldType)? , $($CustomType)?),
                )+
            }

            impl Default for $Register {
                fn default() -> Self {
                    Self {
                        $(
                            $field: default_value!($RegisterType, $($FieldType)?, $($DefaultValue)?),
                        )+
                    }
                }
            }

            impl $Register {
                use crate::macros::registers::default_value;

                const REGISTER_ADDRESS: *const $RegisterType = ($base + $offset) as *const $RegisterType;

                pub unsafe fn read_raw() -> $RegisterType {
                    core::ptr::read_volatile(REGISTER_ADDRESS)
                }

                pub unsafe fn write_raw(data: $RegisterType) {
                    core::ptr::write_volatile(REGISTER_ADDRESS, data)
                }

                fn from_raw(value: $RegisterType) -> Self {
                    Self {
                        $(
                            $field: ((value & mask!($bits$(..$bits_end)?)) >> $bits) as field_type!($RegisterType, $($FieldType)?),
                        )+
                    }
                }

                fn into_raw(self) -> $RegisterType {
                    0 $(
                        | (((self.$field as $RegisterType) << $bits) & mask!($bits$(..$bits_end)?))
                    )+
                }

                pub fn read() -> Self {
                    from_raw(unsafe {read_raw()})
                }

                pub fn write(self) {
                    let data = into_raw();
                    unsafe {
                        write_raw(data);
                    }
                }
            }
            )*
        }
    }
}

pub(crate) use define_registers;