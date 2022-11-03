use core::arch::asm;

//FIXME: remove this

pub struct _SP;
impl _SP {
    /// Returns the current stack pointer.
    #[inline(always)]
    pub fn get(&self) -> u64 {
        let rtn: u64;
        unsafe {
            asm!("mov {r}, sp",
            r = out(reg) rtn,
            );
        }
        rtn
    }

    /// Set the current stack pointer with an passed argument.
    #[inline(always)]
    pub unsafe fn set(&self, stack: u64) {
        asm!("mov sp, {s}",
        s = in(reg) stack,
        );
    }
}
pub static SP: _SP = _SP {};