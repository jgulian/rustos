use core::any::TypeId;

struct Floating;

struct PullDown;

struct PullUp;



#[inline(always)]
fn set_gpio_function<const P: usize, const F: u32>() {
    let type_id = TypeId::of::<T>();

}


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
