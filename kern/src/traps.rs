mod frame;
mod syndrome;
mod syscall;

pub mod irq;

use core::convert::TryInto;
use core::fmt;
use core::fmt::Formatter;
pub use self::frame::TrapFrame;

use pi::interrupt::{Controller, Interrupt};
use crate::{IRQ, kprintln, Shell};

use self::syndrome::Syndrome;
use self::syscall::handle_syscall;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Kind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Source {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Info")
            .field("source", &self.source)
            .field("kind", &self.kind)
            .finish()
    }
}

/// This function is called when an exception occurs. The `info` parameter
/// specifies the source and kind of exception that has occurred. The `esr` is
/// the value of the exception syndrome register. Finally, `tf` is a pointer to
/// the trap frame for the exception.
#[no_mangle]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    let syndrome = Syndrome::from(esr);

    kprintln!("handle_exception {}", info);
    kprintln!("{}", syndrome);

    match info.kind {
        Kind::Synchronous => {
            match syndrome {
                Syndrome::Brk(_) => {
                    tf.elr += 4;
                },
                Syndrome::Svc(s) => {
                    handle_syscall(s, tf);
                }
                _ => {}
            }
        }
        Kind::Irq => {
            let controller = Controller::new();
            for int in Interrupt::iter() {
                if controller.is_pending(*int) {
                    IRQ.invoke(*int, tf);
                }
            }
        }
        _ => {}
    }
}