macro_rules! define_state {
    ($((($data:ident, $T:ty),))*, $($value:ident,)*) => {

    }
}

macro_rules! define_state_machine {
    ($name:ident, [$(($data:ident, $T:ty)$(,)?)*], {$($state:ident: [$($value:expr$(,)?)*]$(,)?)*}) => {
        pub trait $name {
            $(
            const $data: $T;
            )*
        }



        $(
        pub struct $state;

        define_state!($(($data, $T),)*, $(value)*)

        //impl $name for $state {
        //    $(
        //    const $data: $T = $value;
        //    )*
        //}
        )*
    }
}

pub(crate) use define_state_machine;

define_state_machine!(GpioFunction, [(FunctionSelect, u8)], {
    InputFunction: [0b000],
    OutputFunction: [0b001],
    Alt0Function: [0b010],
    Alt1Function: [0b011],
    Alt2Function: [0b100],
    Alt3Function: [0b101],
    Alt4Function: [0b110],
    Alt5Function: [0b111],
});