
pub trait ExceptionRegistrar {
    fn new() -> Self where Self: Sized;
    
    fn on_exception(&mut self, exception_type: ExceptionType, function: Box<dyn FnMut()>);
}

pub enum ExceptionType {
    Syscall,
    PageFault,
    Timer,
}
