mod frame;
mod syndrome;
mod syscall;

pub mod irq;

use core::fmt;
use core::fmt::Formatter;
use aarch64::enable_fiq_interrupt;
pub use self::frame::TrapFrame;

use pi::interrupt::{Controller, Interrupt};
use crate::GLOABAL_IRQ;
use pi::local_interrupt::{LocalController, LocalInterrupt};
use crate::multiprocessing::per_core::local_irq;

use self::syndrome::Syndrome;
use self::syscall::handle_syscall;
use crate::traps::irq::IrqHandlerRegistry;

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

    info!("handle_exception {}: {}", info, syndrome);

    match info.kind {
        Kind::Synchronous => {
            enable_fiq_interrupt();
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
            enable_fiq_interrupt();
            let core = aarch64::affinity();
            if core == 0 {
                let global_controller = Controller::new();
                for global_interrupt in Interrupt::iter() {
                    if global_controller.is_pending(global_interrupt) {
                        GLOABAL_IRQ.invoke(global_interrupt, tf);
                    }
                }
            }

            let local_controller = LocalController::new(core);
            for local_int in LocalInterrupt::iter() {
                if local_controller.is_pending(local_int) {
                    local_irq().invoke(local_int, tf);
                }
            }
        }
        Kind::Fiq => {

        },
        _ => {}
    }
}
