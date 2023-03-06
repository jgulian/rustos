
use core::fmt;
use core::fmt::Formatter;

use aarch64::enable_fiq_interrupt;
use kernel_api::{OsError, OsResult};
use pi::interrupt::{Controller, Interrupt};
use pi::local_interrupt::{LocalController, LocalInterrupt};

use crate::{GLOABAL_IRQ, SCHEDULER};
use crate::multiprocessing::per_core::local_irq;
use crate::process::State;
use crate::traps::irq::IrqHandlerRegistry;
use crate::traps::memory::handle_memory_abort;


pub use self::frame::TrapFrame;
use self::syndrome::Syndrome;
use self::syscall::handle_syscall;

mod frame;
mod memory;
mod syndrome;
mod syscall;

pub mod irq;

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
pub extern "C" fn receive_exception(info: Info, esr: u32, trap_frame: &mut TrapFrame) {
    let syndrome = Syndrome::from(esr);

    //TODO: default to killing processes if they create exceptions I don't handle.

    match handle_exception(info, syndrome, trap_frame) {
        Ok(_) => {}
        Err(_) => {
            SCHEDULER.switch(State::Dead, trap_frame);
        }
    }
}

fn handle_exception(info: Info, syndrome: Syndrome, trap_frame: &mut TrapFrame) -> OsResult<()> {
    match info.kind {
        Kind::Synchronous => {
            enable_fiq_interrupt();
            match syndrome {
                Syndrome::Brk(_) => {
                    trap_frame.elr += 4;
                    Ok(())
                }
                Syndrome::Svc(s) => {
                    handle_syscall(s, trap_frame);
                    Ok(())
                },
                Syndrome::DataAbort(abort_data) =>
                    handle_memory_abort(trap_frame, abort_data),
                Syndrome::InstructionAbort(abort_data) =>
                    handle_memory_abort(trap_frame, abort_data),
                _ => Err(OsError::Unknown),
            }
        }
        Kind::Irq => {
            enable_fiq_interrupt();
            let core = aarch64::affinity();
            if core == 0 {
                let global_controller = Controller::new();
                for global_interrupt in Interrupt::iter() {
                    if global_controller.is_pending(global_interrupt) {
                        GLOABAL_IRQ.invoke(global_interrupt, trap_frame);
                    }
                }
            }

            let local_controller = LocalController::new(core);
            for local_int in LocalInterrupt::iter() {
                if local_controller.is_pending(local_int) {
                    local_irq().invoke(local_int, trap_frame);
                }
            }

            Ok(())
        }
        Kind::Fiq => Err(OsError::Unknown),
        _ => Err(OsError::Unknown)
    }
}
