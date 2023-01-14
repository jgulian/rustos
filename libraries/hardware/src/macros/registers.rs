//TODO: there is no readwrite, add this
//TODO: do less writes with or

macro_rules! mask {
    ($bit:literal) => {
        (0b1 << $bit)
    };
    ($bit_beg:literal..$bit_end:literal) => {
        ((1 << ($bit_end + 1)) - (1 << $bit_beg))
    };
}

pub(crate) use mask;

macro_rules! field_type {
    ($First:ty,) => { $First };
    ($($First:ty)?, $Second:ty) => { $Second };
    ($First:ty,,) => { $First };
    ($($First:ty)?, $Second:ty,) => { $Second };
    ($($First:ty)?, $($Second:ty)?,$Third:ty) => { $Third };
}

pub(crate) use field_type;

macro_rules! default_value {
    ($RegisterType:ident,,) => {$RegisterType::default()};
    ($RegisterType:ident,$FieldType:ident,) => {$FieldType::default()};
    ($RegisterType:ident,$($FieldType:ident)?,$default:expr) => {$default};
}

pub(crate) use default_value;

macro_rules! into_field_type {
    ($data:expr, bool,,) => { $data != 0 };
    ($data:expr, $First:ty,bool,) => { $data != 0 };
    ($data:expr, $First:ty,$($Second:ty)?,) => { $data as field_type!($First, $($Second)?) };
    ($data:expr, $($First:ty)?, $($Second:ty)?,$Third:ty) => { <$Third>::from($data) };
}


pub(crate) use into_field_type;

macro_rules! define_registers {
    ($module:ident, $base:literal, [
        $(
            ($Register:ident, $RegisterType:ident, $offset:literal): [
                $(($field:ident, $bits:literal$(..$bits_end:literal)?, $ReadWrite:ident, {
                    $(FieldType: $FieldType:ident,)?
                    $(CustomType: $CustomType:ident {$($CTypeName:ident = $CTypeValue:literal,)+},)?
                    $(DefaultValue: $DefaultValue:expr,)?
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

                    impl From<$RegisterType> for $CustomType {
                        fn from(item: $RegisterType) -> Self {
                            match item {
                                $(
                                    $CTypeValue => $CustomType::$CTypeName,
                                )+
                                _ => {
                                    // TODO: fail softly?
                                    panic!("Invalid hardware description");
                                }
                            }
                        }
                    }
                )?
            )*

            pub struct $Register {
                $(
                    pub $field: field_type!($RegisterType, $($FieldType)? , $($CustomType)?),
                )*
            }

            impl Default for $Register {
                fn default() -> Self {
                    Self {
                        $(
                            $field: default_value!($RegisterType, $($FieldType)?, $($DefaultValue)?),
                        )*
                    }
                }
            }

            impl $Register {
                const REGISTER_ADDRESS: *mut $RegisterType = ($base + $offset) as *mut $RegisterType;

                pub unsafe fn read_raw() -> $RegisterType {
                    core::ptr::read_volatile(Self::REGISTER_ADDRESS)
                }

                pub unsafe fn write_raw(data: $RegisterType) {
                    core::ptr::write_volatile(Self::REGISTER_ADDRESS, data)
                }

                fn from_raw(value: $RegisterType) -> Self {
                    Self {
                        $(
                            $field: into_field_type!(((value & mask!($bits$(..$bits_end)?)) >> $bits), $RegisterType, $($FieldType)?, $($CustomType)?),
                        )*
                    }
                }

                fn into_raw(self) -> $RegisterType {
                    0 $(
                        | (((self.$field as $RegisterType) << $bits) & mask!($bits$(..$bits_end)?))
                    )*
                }

                pub fn read() -> Self {
                    Self::from_raw(unsafe {Self::read_raw()})
                }

                pub fn write(self) {
                    let data = self.into_raw();
                    unsafe {
                        Self::write_raw(data);
                    }
                }
            }
            )*
        }
    };
}

pub(crate) use define_registers;