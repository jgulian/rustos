macro_rules! define_state {
    ($((($data:ident, $T:ty),))*, $($value:ident,)*) => {

    }
}

macro_rules! define_state_machine {
    ($name:ident, [$(($data:ident, $T:ty)$(,)?)*], {$($state:ident: {
        $($state_data:ident: $data_type:ty = $value:expr$(,)?)*}$(,)?)*
    }) => {
        pub trait $name {
            $(
            const $data: $T;
            )*
        }

        $(
        pub struct $state;
        
        impl $name for $state {
            $(
            const $state_data: $data_type = $value;
            )*
        }
        )*
    }
}

pub(crate) use define_state_machine;