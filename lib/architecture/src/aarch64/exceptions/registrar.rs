use alloc::boxed::Box;
use crate::primitives;
use crate::primitives::exception::ExceptionType;

struct Aarch64ExceptionRegistrar;

impl primitives::ExceptionRegistrar for Aarch64ExceptionRegistrar {
    fn new() -> Self where Self: Sized {
        Aarch64ExceptionRegistrar
    }

    fn on_exception(&mut self, exception_type: ExceptionType, function: Box<dyn FnMut()>) {
        todo!()
    }
}