use core::fmt;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    pub xzr: u64,
    pub xs: [u64; 31],
    pub qs: [f64; 32],
    pub tpidr: u64,
    pub sp: u64,
    pub spsr: u64,
    pub elr: u64,
}

