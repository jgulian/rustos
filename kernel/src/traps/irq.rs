use alloc::boxed::Box;
use core::ops::Index;

use pi::interrupt::Interrupt;
use pi::local_interrupt::LocalInterrupt;

use crate::mutex::{Mutex, MutexGuard};
use crate::traps::TrapFrame;

// Programmer Guide Chapter 10
// AArch64 Exception Handling
pub type IrqHandler = Box<dyn FnMut(&mut TrapFrame) + Send>;
type IrqHandlerMutex = Mutex<Option<IrqHandler>>;

type GlobalIrqHandlers = [IrqHandlerMutex; Interrupt::MAX];
type LocalIrqHandlers = [IrqHandlerMutex; LocalInterrupt::MAX];

/// Global IRQ handler registry.
pub struct GlobalIrq(GlobalIrqHandlers);
/// Local (per-core) IRQ handler registry. (QA7: Chapter 4)
pub struct LocalIrq(LocalIrqHandlers);
/// Global FIQ handler registry. Our kerenl supports only one FIQ interrupt.
pub struct Fiq(IrqHandlerMutex);

impl GlobalIrq {
    pub const fn new() -> GlobalIrq {
        GlobalIrq([
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
        ])
    }
}

impl LocalIrq {
    pub const fn new() -> LocalIrq {
        LocalIrq([
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
            Mutex::new(None),
        ])
    }
}

impl Fiq {
    pub const fn new() -> Fiq {
        Fiq(Mutex::new(None))
    }
}

impl Index<Interrupt> for GlobalIrq {
    type Output = IrqHandlerMutex;

    fn index(&self, int: Interrupt) -> &IrqHandlerMutex {
        use Interrupt::*;
        let index = match int {
            Timer1 => 0,
            Timer3 => 1,
            Usb => 2,
            Gpio0 => 3,
            Gpio1 => 4,
            Gpio2 => 5,
            Gpio3 => 6,
            Uart => 7,
        };
        &self.0[index]
    }
}

impl Index<LocalInterrupt> for LocalIrq {
    type Output = IrqHandlerMutex;

    fn index(&self, int: LocalInterrupt) -> &IrqHandlerMutex {
        use LocalInterrupt::*;
        &self.0[int as usize]
    }
}

impl Index<()> for Fiq {
    type Output = IrqHandlerMutex;

    fn index(&self, _: ()) -> &IrqHandlerMutex {
        &self.0
    }
}

/// A trait that defines the behavior of an IRQ (and FIQ) handler registry.
pub trait IrqHandlerRegistry<I> {
    fn register(&self, int: I, handler: IrqHandler);
    fn invoke(&self, int: I, tf: &mut TrapFrame);
}

/// A blanket implementation of `IrqHandlerRegistry` trait for all indexable
/// struct that returns `IrqHandlerMutex`.
impl<I, T> IrqHandlerRegistry<I> for T
where
    T: Index<I, Output = IrqHandlerMutex>,
{
    /// Register an irq handler for an interrupt.
    /// The caller should assure that `initialize()` has been called before calling this function.
    fn register(&self, int: I, handler: IrqHandler) {
        self.index(int).lock().replace(handler);
    }

    /// Executes an irq handler for the givven interrupt.
    /// The caller should assure that `initialize()` has been called before calling this function.
    fn invoke(&self, int: I, tf: &mut TrapFrame) {
        match self.index(int).lock().as_mut() {
            Some(handler) => {
                (**handler)(tf);
            }
            _ => {}
        }
    }
}
