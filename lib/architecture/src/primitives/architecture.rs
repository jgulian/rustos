use super::memory;

pub trait Architecture {
    type MemoryManager: memory::MemoryManager;
    //type ExceptionRegistrar
}
