use crate::macros::registers::define_registers;

define_registers!(uart_registers, 0x3f20_0000, [
    (AuxIrq, u32, 0x00): [
        (mini_uart_irq, 0, Read, {FieldType: bool, DefaultValue: false,}),
        (spi_one_irq, 1, Read, {FieldType: bool, DefaultValue: false,}),
        (spi_two_irq, 2, Read, {FieldType: bool, DefaultValue: false,}),
    ],
]);

macro_rules! gpio {
    ($GpioPin:ident, $gpio_pin:ident, $pin:literal, $pull:ty, [
        $($function:ty: $id:ident,)+
    ]) => {
        pub mod $gpio_pin {
            use core::marker::PhantomData;

            #[warn(unused_imports)]
            use super::{Floating, $pull, InputFunction, OutputFunction, $($function,)*
            set_gpio_function};

            pub struct $GpioPin<MODE> {
                _mode: PhantomData<MODE>,
            }

            impl<MODE> $GpioPin<MODE> {
                pub fn input(self) -> $GpioPin<InputFunction> {
                    self.transition()
                }

                pub fn output(self) -> $GpioPin<OutputFunction> {
                    self.transition()
                }

                $(
                pub fn $id(self) -> $GpioPin<$function> {
                    self.transition()
                }
                )*

                #[inline(always)]
                fn transition<T: GpioFunction>(self) -> $GpioPin<T> {
                    set_gpio_function::<$pin, T>();
                    $GpioPin {
                        _mode: PhantomData,
                    }
                }
            }

            impl $GpioPin<InputFunction> {

            }

            impl $GpioPin<OutputFunction> {

            }
        }
    }
}

gpio!(Gpio0, gpio_0, 0, PullHigh, [
    Alt0Function: sda0,
    Alt1Function: sa5,
]);

gpio!(Gpio14, gpio_14, 14, PullHigh, [
    Alt0Function: txd0,
    Alt1Function: sd6,
    Alt5Function: txd1,
]);

gpio!(Gpio15, gpio_15, 15, PullHigh, [
    Alt0Function: rxd0,
    Alt1Function: sd7,
    Alt5Function: rxd1,
]);
