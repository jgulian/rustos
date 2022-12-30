use core::time::Duration;

use shim::const_assert_size;
use volatile::prelude::*;
use volatile::Volatile;

const INT_BASE: usize = 0x4000_0000;

/// Core interrupt sources (QA7: 4.10)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterrupt {
    CntPsIrq = 0,
    CntPnsIrq = 1,
    CntHpIrq = 2,
    CntVIrq = 3,
    Mailbox1 = 4,
    Mailbox2 = 5,
    Mailbox3 = 6,
    Mailbox4 = 7,
    Gpu = 8,
    Pmu = 9,
    Axi = 10,
    Timer = 11,
    Unknown = 12,
}

impl LocalInterrupt {
    pub const MAX: usize = 12;

    pub fn iter() -> impl Iterator<Item=LocalInterrupt> {
        (0..LocalInterrupt::MAX).map(|n| LocalInterrupt::from(n))
    }
}

impl From<usize> for LocalInterrupt {
    fn from(irq: usize) -> LocalInterrupt {
        match irq {
            0 => LocalInterrupt::CntPsIrq,
            1 => LocalInterrupt::CntPnsIrq,
            2 => LocalInterrupt::CntHpIrq,
            3 => LocalInterrupt::CntVIrq,
            4 => LocalInterrupt::Mailbox1,
            5 => LocalInterrupt::Mailbox2,
            6 => LocalInterrupt::Mailbox3,
            7 => LocalInterrupt::Mailbox4,
            8 => LocalInterrupt::Gpu,
            9 => LocalInterrupt::Pmu,
            10 => LocalInterrupt::Axi,
            11 => LocalInterrupt::Timer,
            _ => LocalInterrupt::Unknown,
        }
    }
}

/// BCM2837 Local Peripheral Registers (QA7: Chapter 4)
#[repr(C)]
pub struct Registers {
    control: Volatile<u32>,
    __reserved1: u32,
    core_timer_prescaler: Volatile<u32>,
    gpu_interrupt_router: Volatile<u32>,
    pm_interrupt_set: Volatile<u32>,
    pm_interrupt_clear: Volatile<u32>,
    __reserved2: u32,
    core_timer_ls: Volatile<u32>,
    core_timer_ms: Volatile<u32>,
    local_interrupt_routing: Volatile<u32>,
    __reserved3: u32,
    //TODO: read documentation more, was this removed?
    axi_outstanding_counters: Volatile<u32>,
    axi_outstanding_irq: Volatile<u32>,
    local_timer_control: Volatile<u32>,
    local_timer_write: Volatile<u32>,
    __reserved4: u32,
    core_timer: [Volatile<u32>; 4],
    core_mailbox: [Volatile<u32>; 4],
    core_irq_source: [Volatile<u32>; 4],
    core_fiq_source: [Volatile<u32>; 4],
}

const_assert_size!(Registers, 0x80);

pub struct LocalController {
    core: usize,
    pub registers: &'static mut Registers,
}

impl LocalController {
    /// Returns a new handle to the interrupt controller.
    pub fn new(core: usize) -> LocalController {
        LocalController {
            core: core,
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    pub fn enable_local_timer(&mut self) {
        unsafe {
            aarch64::CNTP_CTL_EL0.set(aarch64::CNTP_CTL_EL0.get() |
                aarch64::CNTP_CTL_EL0::ENABLE | aarch64::CNTP_CTL_EL0::IMASK);
            self.registers.core_timer[self.core].write(0b10);
            self.registers.core_irq_source[self.core].write(0b10);
        }
    }

    pub fn is_pending(&self, int: LocalInterrupt) -> bool {
        self.registers.core_irq_source[self.core].read() & (1 << (int as u32)) > 0
    }

    pub fn tick_in(&mut self, t: Duration) {
        let additional_time = (t.as_nanos() * unsafe { aarch64::CNTFRQ_EL0.get() as u128 } / 1_000_000_000u128) as u64;
        unsafe {
            aarch64::CNTP_TVAL_EL0.set(additional_time);
            aarch64::CNTP_CTL_EL0.set(aarch64::CNTP_CTL_EL0.get() & (!aarch64::CNTP_CTL_EL0::IMASK));
        }
    }
}

pub fn local_tick_in(core: usize, t: Duration) {
    LocalController::new(core).tick_in(t)
}
