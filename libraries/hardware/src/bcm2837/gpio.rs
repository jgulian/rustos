use core::marker::PhantomData;

use shim::io;
use crate::macros::registers::define_registers;

define_registers!(gpio_registers, 0x3f20_0000, [
    (FunctionSelect0, u32, 0x00): [
        (gpio_0, 0..2, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_1, 3..5, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_2, 6..8, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_3, 9..11, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_4, 12..14, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_5, 15..17, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_6, 18..20, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_7, 21..23, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_8, 24..26, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_9, 27..29, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
    ],
    (FunctionSelect1, u32, 0x04): [
        (gpio_10, 0..2, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_11, 3..5, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_12, 6..8, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_13, 9..11, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_14, 12..14, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_15, 15..17, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_16, 18..20, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_17, 21..23, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_18, 24..26, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_19, 27..29, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
    ],
    (FunctionSelect2, u32, 0x08): [
        (gpio_20, 0..2, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_21, 3..5, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_22, 6..8, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_23, 9..11, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_24, 12..14, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_25, 15..17, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_26, 18..20, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_27, 21..23, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_28, 24..26, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_29, 27..29, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
    ],
    (FunctionSelect3, u32, 0x0c): [
        (gpio_30, 0..2, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_31, 3..5, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_32, 6..8, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_33, 9..11, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_34, 12..14, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_35, 15..17, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_36, 18..20, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_37, 21..23, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_38, 24..26, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_39, 27..29, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
    ],
    (FunctionSelect4, u32, 0x10): [
        (gpio_40, 0..2, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_41, 3..5, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_42, 6..8, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_43, 9..11, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_44, 12..14, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_45, 15..17, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_46, 18..20, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_47, 21..23, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_48, 24..26, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_49, 27..29, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
    ],
    (FunctionSelect5, u32, 0x14): [
        (gpio_50, 0..2, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_51, 3..5, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_52, 6..8, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
        (gpio_53, 9..11, ReadWrite, {FieldType: u8, DefaultValue: 0b000,}),
    ],
    (OutputSet0, u32, 0x1c): [
        (gpio_0, 0, Read, {FieldType: bool,}),
        (gpio_1, 1, Read, {FieldType: bool,}),
        (gpio_2, 2, Read, {FieldType: bool,}),
        (gpio_3, 3, Read, {FieldType: bool,}),
        (gpio_4, 4, Read, {FieldType: bool,}),
        (gpio_5, 5, Read, {FieldType: bool,}),
        (gpio_6, 6, Read, {FieldType: bool,}),
        (gpio_7, 7, Read, {FieldType: bool,}),
        (gpio_8, 8, Read, {FieldType: bool,}),
        (gpio_9, 9, Read, {FieldType: bool,}),
        (gpio_10, 10, Read, {FieldType: bool,}),
        (gpio_11, 11, Read, {FieldType: bool,}),
        (gpio_12, 12, Read, {FieldType: bool,}),
        (gpio_13, 13, Read, {FieldType: bool,}),
        (gpio_14, 14, Read, {FieldType: bool,}),
        (gpio_15, 15, Read, {FieldType: bool,}),
        (gpio_16, 16, Read, {FieldType: bool,}),
        (gpio_17, 17, Read, {FieldType: bool,}),
        (gpio_18, 18, Read, {FieldType: bool,}),
        (gpio_19, 19, Read, {FieldType: bool,}),
        (gpio_20, 20, Read, {FieldType: bool,}),
        (gpio_21, 21, Read, {FieldType: bool,}),
        (gpio_22, 22, Read, {FieldType: bool,}),
        (gpio_23, 23, Read, {FieldType: bool,}),
        (gpio_24, 24, Read, {FieldType: bool,}),
        (gpio_25, 25, Read, {FieldType: bool,}),
        (gpio_26, 26, Read, {FieldType: bool,}),
        (gpio_27, 27, Read, {FieldType: bool,}),
        (gpio_28, 28, Read, {FieldType: bool,}),
        (gpio_29, 29, Read, {FieldType: bool,}),
        (gpio_30, 30, Read, {FieldType: bool,}),
        (gpio_31, 31, Read, {FieldType: bool,}),
    ],
    (OutputSet1, u32, 0x20): [
        (gpio_32, 0, Read, {FieldType: bool,}),
        (gpio_33, 1, Read, {FieldType: bool,}),
        (gpio_34, 2, Read, {FieldType: bool,}),
        (gpio_35, 3, Read, {FieldType: bool,}),
        (gpio_36, 4, Read, {FieldType: bool,}),
        (gpio_37, 5, Read, {FieldType: bool,}),
        (gpio_38, 6, Read, {FieldType: bool,}),
        (gpio_39, 7, Read, {FieldType: bool,}),
        (gpio_40, 8, Read, {FieldType: bool,}),
        (gpio_41, 9, Read, {FieldType: bool,}),
        (gpio_42, 10, Read, {FieldType: bool,}),
        (gpio_43, 11, Read, {FieldType: bool,}),
        (gpio_44, 12, Read, {FieldType: bool,}),
        (gpio_45, 13, Read, {FieldType: bool,}),
        (gpio_46, 14, Read, {FieldType: bool,}),
        (gpio_47, 15, Read, {FieldType: bool,}),
        (gpio_48, 16, Read, {FieldType: bool,}),
        (gpio_49, 17, Read, {FieldType: bool,}),
        (gpio_50, 18, Read, {FieldType: bool,}),
        (gpio_51, 19, Read, {FieldType: bool,}),
        (gpio_52, 20, Read, {FieldType: bool,}),
        (gpio_53, 21, Read, {FieldType: bool,}),
    ],
    (OutputClear0, u32, 0x1c): [
        (gpio_0, 0, Read, {FieldType: bool,}),
        (gpio_1, 1, Read, {FieldType: bool,}),
        (gpio_2, 2, Read, {FieldType: bool,}),
        (gpio_3, 3, Read, {FieldType: bool,}),
        (gpio_4, 4, Read, {FieldType: bool,}),
        (gpio_5, 5, Read, {FieldType: bool,}),
        (gpio_6, 6, Read, {FieldType: bool,}),
        (gpio_7, 7, Read, {FieldType: bool,}),
        (gpio_8, 8, Read, {FieldType: bool,}),
        (gpio_9, 9, Read, {FieldType: bool,}),
        (gpio_10, 10, Read, {FieldType: bool,}),
        (gpio_11, 11, Read, {FieldType: bool,}),
        (gpio_12, 12, Read, {FieldType: bool,}),
        (gpio_13, 13, Read, {FieldType: bool,}),
        (gpio_14, 14, Read, {FieldType: bool,}),
        (gpio_15, 15, Read, {FieldType: bool,}),
        (gpio_16, 16, Read, {FieldType: bool,}),
        (gpio_17, 17, Read, {FieldType: bool,}),
        (gpio_18, 18, Read, {FieldType: bool,}),
        (gpio_19, 19, Read, {FieldType: bool,}),
        (gpio_20, 20, Read, {FieldType: bool,}),
        (gpio_21, 21, Read, {FieldType: bool,}),
        (gpio_22, 22, Read, {FieldType: bool,}),
        (gpio_23, 23, Read, {FieldType: bool,}),
        (gpio_24, 24, Read, {FieldType: bool,}),
        (gpio_25, 25, Read, {FieldType: bool,}),
        (gpio_26, 26, Read, {FieldType: bool,}),
        (gpio_27, 27, Read, {FieldType: bool,}),
        (gpio_28, 28, Read, {FieldType: bool,}),
        (gpio_29, 29, Read, {FieldType: bool,}),
        (gpio_30, 30, Read, {FieldType: bool,}),
        (gpio_31, 31, Read, {FieldType: bool,}),
    ],
    (OutputClear1, u32, 0x20): [
        (gpio_32, 0, Read, {FieldType: bool,}),
        (gpio_33, 1, Read, {FieldType: bool,}),
        (gpio_34, 2, Read, {FieldType: bool,}),
        (gpio_35, 3, Read, {FieldType: bool,}),
        (gpio_36, 4, Read, {FieldType: bool,}),
        (gpio_37, 5, Read, {FieldType: bool,}),
        (gpio_38, 6, Read, {FieldType: bool,}),
        (gpio_39, 7, Read, {FieldType: bool,}),
        (gpio_40, 8, Read, {FieldType: bool,}),
        (gpio_41, 9, Read, {FieldType: bool,}),
        (gpio_42, 10, Read, {FieldType: bool,}),
        (gpio_43, 11, Read, {FieldType: bool,}),
        (gpio_44, 12, Read, {FieldType: bool,}),
        (gpio_45, 13, Read, {FieldType: bool,}),
        (gpio_46, 14, Read, {FieldType: bool,}),
        (gpio_47, 15, Read, {FieldType: bool,}),
        (gpio_48, 16, Read, {FieldType: bool,}),
        (gpio_49, 17, Read, {FieldType: bool,}),
        (gpio_50, 18, Read, {FieldType: bool,}),
        (gpio_51, 19, Read, {FieldType: bool,}),
        (gpio_52, 20, Read, {FieldType: bool,}),
        (gpio_53, 21, Read, {FieldType: bool,}),
    ],
    (InputLevel0, u32, 0x1c): [
        (gpio_0, 0, Read, {FieldType: bool,}),
        (gpio_1, 1, Read, {FieldType: bool,}),
        (gpio_2, 2, Read, {FieldType: bool,}),
        (gpio_3, 3, Read, {FieldType: bool,}),
        (gpio_4, 4, Read, {FieldType: bool,}),
        (gpio_5, 5, Read, {FieldType: bool,}),
        (gpio_6, 6, Read, {FieldType: bool,}),
        (gpio_7, 7, Read, {FieldType: bool,}),
        (gpio_8, 8, Read, {FieldType: bool,}),
        (gpio_9, 9, Read, {FieldType: bool,}),
        (gpio_10, 10, Read, {FieldType: bool,}),
        (gpio_11, 11, Read, {FieldType: bool,}),
        (gpio_12, 12, Read, {FieldType: bool,}),
        (gpio_13, 13, Read, {FieldType: bool,}),
        (gpio_14, 14, Read, {FieldType: bool,}),
        (gpio_15, 15, Read, {FieldType: bool,}),
        (gpio_16, 16, Read, {FieldType: bool,}),
        (gpio_17, 17, Read, {FieldType: bool,}),
        (gpio_18, 18, Read, {FieldType: bool,}),
        (gpio_19, 19, Read, {FieldType: bool,}),
        (gpio_20, 20, Read, {FieldType: bool,}),
        (gpio_21, 21, Read, {FieldType: bool,}),
        (gpio_22, 22, Read, {FieldType: bool,}),
        (gpio_23, 23, Read, {FieldType: bool,}),
        (gpio_24, 24, Read, {FieldType: bool,}),
        (gpio_25, 25, Read, {FieldType: bool,}),
        (gpio_26, 26, Read, {FieldType: bool,}),
        (gpio_27, 27, Read, {FieldType: bool,}),
        (gpio_28, 28, Read, {FieldType: bool,}),
        (gpio_29, 29, Read, {FieldType: bool,}),
        (gpio_30, 30, Read, {FieldType: bool,}),
        (gpio_31, 31, Read, {FieldType: bool,}),
    ],
    (InputLevel1, u32, 0x20): [
        (gpio_32, 0, Read, {FieldType: bool,}),
        (gpio_33, 1, Read, {FieldType: bool,}),
        (gpio_34, 2, Read, {FieldType: bool,}),
        (gpio_35, 3, Read, {FieldType: bool,}),
        (gpio_36, 4, Read, {FieldType: bool,}),
        (gpio_37, 5, Read, {FieldType: bool,}),
        (gpio_38, 6, Read, {FieldType: bool,}),
        (gpio_39, 7, Read, {FieldType: bool,}),
        (gpio_40, 8, Read, {FieldType: bool,}),
        (gpio_41, 9, Read, {FieldType: bool,}),
        (gpio_42, 10, Read, {FieldType: bool,}),
        (gpio_43, 11, Read, {FieldType: bool,}),
        (gpio_44, 12, Read, {FieldType: bool,}),
        (gpio_45, 13, Read, {FieldType: bool,}),
        (gpio_46, 14, Read, {FieldType: bool,}),
        (gpio_47, 15, Read, {FieldType: bool,}),
        (gpio_48, 16, Read, {FieldType: bool,}),
        (gpio_49, 17, Read, {FieldType: bool,}),
        (gpio_50, 18, Read, {FieldType: bool,}),
        (gpio_51, 19, Read, {FieldType: bool,}),
        (gpio_52, 20, Read, {FieldType: bool,}),
        (gpio_53, 21, Read, {FieldType: bool,}),
    ],
]);

pub trait InputPin {
    fn is_low() -> io::Result<bool>;
    fn is_high() -> io::Result<bool>;
}

pub trait OutputPin {
    fn pull_low() -> io::Result<()>;
    fn pull_high() -> io::Result<()>;
}

mod gpio_functions {
    use crate::macros::states::define_state_machine;
    define_state_machine!(GpioFunction, [(FunctionSelect, u8)], {
        Uninitialized: {FunctionSelect: u8 = 0xff},
        InputFunction: {FunctionSelect: u8 = 0b000},
        OutputFunction: {FunctionSelect: u8 = 0b001},
        Alt0Function: {FunctionSelect: u8 = 0b010},
        Alt1Function: {FunctionSelect: u8 = 0b011},
        Alt2Function: {FunctionSelect: u8 = 0b100},
        Alt3Function: {FunctionSelect: u8 = 0b101},
        Alt4Function: {FunctionSelect: u8 = 0b110},
        Alt5Function: {FunctionSelect: u8 = 0b111},
    });
}

use gpio_functions::*;

macro_rules! gpio {
    ($GpioPin:ident, $gpio_pin:ident,
    [$function_select:ident, $set:ident, $clear:ident, $level:ident $(,)?],
    [
        $($function:ty: $id:ident,)+
    ]) => {
        pub struct $GpioPin<T: GpioFunction> {
            _mode: PhantomData<T>,
        }

        impl $GpioPin<Uninitialized> {
            pub fn new() -> Self {
                Self {
                    _mode: PhantomData,
                }
            }

            #[inline(always)]
            pub fn input(self) -> $GpioPin<InputFunction> {
                self.transition()
            }

            #[inline(always)]
            pub fn output(self) -> $GpioPin<OutputFunction> {
                self.transition()
            }

            $(
            #[inline(always)]
            pub fn $id(self) -> $GpioPin<$function> {
                self.transition()
            }
            )*

            #[inline(always)]
            fn transition<T: GpioFunction>(self) -> $GpioPin<T> {
                let mut register = gpio_registers::$function_select::read();
                register.$gpio_pin = T::FunctionSelect;
                register.write();

                $GpioPin {
                    _mode: PhantomData,
                }
            }
        }

        impl InputPin for $GpioPin<InputFunction> {
            fn is_low() -> io::Result<bool> {
                use gpio_registers::$level;
                let $level {$gpio_pin, ..} = $level::read();
                Ok(!$gpio_pin)
            }
            fn is_high() -> io::Result<bool> {
                use gpio_registers::$level;
                let $level {$gpio_pin, ..} = $level::read();
                Ok($gpio_pin)
            }
        }

        //TODO: doesn't work... it needs or
        //impl OutputPin for $GpioPin<OutputFunction> {
        //    fn pull_low() -> io::Result<()> {
        //        use gpio_registers::$set;
        //        let $level {$gpio_pin, ..} = $level::read();
        //        Ok(!$gpio_pin)
        //    }
        //    fn pull_high() -> io::Result<()> {
        //        use gpio_registers::$clear;
        //        let $level {$gpio_pin, ..} = $level::read();
        //        Ok($gpio_pin)
        //    }
        //}
    }
}

gpio!(Gpio0, gpio_0, [FunctionSelect0, OutputSet0, OutputClear0, InputLevel0], [
    Alt0Function: sda0,
    Alt1Function: sa5,
]);

gpio!(Gpio14, gpio_14, [FunctionSelect1, OutputSet0, OutputClear0, InputLevel0], [
    Alt0Function: txd0,
    Alt1Function: sd6,
    Alt5Function: txd1,
]);

gpio!(Gpio15, gpio_15, [FunctionSelect1, OutputSet0, OutputClear0, InputLevel0], [
    Alt0Function: rxd0,
    Alt1Function: sd7,
    Alt5Function: rxd1,
]);

gpio!(Gpio16, gpio_16, [FunctionSelect1, OutputSet0, OutputClear0, InputLevel0], [
    Alt1Function: sd8,
    Alt3Function: cts0,
    Alt4Function: spi1_ce2_n,
    Alt5Function: cts1,
]);

gpio!(Gpio17, gpio_17, [FunctionSelect1, OutputSet0, OutputClear0, InputLevel0], [
    Alt1Function: sd9,
    Alt3Function: rts0,
    Alt4Function: spi1_ce1_n,
    Alt5Function: rts1,
]);
