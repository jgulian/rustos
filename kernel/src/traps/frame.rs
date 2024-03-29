use core::fmt;
use core::fmt::Formatter;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    pub ttbr0: u64,
    pub ttbr1: u64,
    pub elr: u64,
    pub spsr: u64,
    pub sp: u64,
    pub tpidr: u64,
    pub qs: [u128; 32],
    pub xs: [u64; 31],
    pub xzr: u64,
}

impl fmt::Display for TrapFrame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TrapFrame")
            .field("ttbr0", &self.ttbr0)
            .field("ttbr1", &self.ttbr1)
            .field("xzr", &self.xzr)
            .field("xs", &self.xs)
            .field("qs", &self.qs)
            .field("tpidr", &self.tpidr)
            .field("sp", &self.sp)
            .field("spsr", &self.spsr)
            .field("elr", &self.elr)
            .finish()
    }
}

