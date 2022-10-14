use core::fmt;
use core::fmt::Formatter;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TrapFrame {
    pub elr: u64,
    pub spsr: u64,
    pub sp: u64,
    pub tpidr: u64,
    pub qs: [f64; 32],
    pub xs: [u64; 31],
    pub xzr: u64,
}

impl fmt::Display for TrapFrame {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("TrapFrame")
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

