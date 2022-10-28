use core::ptr::write_volatile;
use core::time::Duration;
use shim::const_assert_size;

use volatile::prelude::*;
use volatile::Volatile;

const INT_BASE: usize = 0x4000_0000;

/// Core interrupt sources (QA7: 4.10)
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LocalInterrupt {
    cntpsirq = 0,
    cntpnsqirq = 1,
    cnthpirq = 2,
    cntvirq = 3,
    mailbox_1 = 4,
    mailbox_2 = 5,
    mailbox_3 = 6,
    mailbox_4 = 7,
    gpu = 8,
    pmu = 9,
    axi = 10,
    timer = 11,
    unknown = 12,
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
            0 => LocalInterrupt::cntpsirq,
            1 => LocalInterrupt::cntpnsqirq,
            2 => LocalInterrupt::cnthpirq,
            3 => LocalInterrupt::cntvirq,
            4 => LocalInterrupt::mailbox_1,
            5 => LocalInterrupt::mailbox_2,
            6 => LocalInterrupt::mailbox_3,
            7 => LocalInterrupt::mailbox_4,
            8 => LocalInterrupt::gpu,
            9 => LocalInterrupt::pmu,
            10 => LocalInterrupt::axi,
            11 => LocalInterrupt::timer,
            _ => LocalInterrupt::unknown,
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
    //TODO: read docs more, was this removed?
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
            let value = (aarch64::CNTP_CTL_EL0.get() | aarch64::CNTP_CTL_EL0::ENABLE)
                & (!aarch64::CNTP_CTL_EL0::IMASK);
            aarch64::CNTP_CTL_EL0.set(aarch64::CNTP_CTL_EL0::ENABLE);
            self.registers.core_timer[self.core].write(0b1);
            self.registers.core_irq_source[self.core].write(0b1);
            self.registers.local_timer_control.write(0b11 << 28 | 20_000_000);
            self.registers.local_timer_write.write(0b11 << 30);
            self.registers.core_timer_prescaler.write(0x8000_0000);

            //asm!("cpsie i");
        }
    }

    pub fn is_pending(&self, int: LocalInterrupt) -> bool {
        self.registers.core_irq_source[self.core].read() & (1 << (int as u32)) > 0
    }

    pub fn tick_in(&mut self, t: Duration) -> u64 {
        unsafe {
            write_volatile((0x4000_0040 + 4 * self.core) as *mut u32, 1 << 1);;
            aarch64::CNTP_TVAL_EL0.set(aarch64::CNTPCT_EL0.get() + 19200000);
            aarch64::CNTP_CTL_EL0.set(1);
        }

        19200000 as u64
    }
}

pub fn local_tick_in(core: usize, t: Duration) -> u64 {
    LocalController::new(core).tick_in(t)
}
