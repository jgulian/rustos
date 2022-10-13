use aarch64::ESR_EL1;
use crate::traps::syndrome::Fault::{AccessFlag, AddressSize, Alignment, Permission, TlbConflict, Translation};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Fault {
    AddressSize,
    Translation,
    AccessFlag,
    Permission,
    Alignment,
    TlbConflict,
    Other(u8),
}

impl From<u32> for Fault {
    fn from(val: u32) -> Fault {
        match val {
            0b000000..=0b000011 => AddressSize,
            0b000100..=0b000111 => Translation,
            0b001001..=0b001011 => AccessFlag,
            0b001101..=0b001111 => Permission,
            100001 => Alignment,
            110000 => TlbConflict,
            _ => Fault::Other(val as u8)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Syndrome {
    Unknown,
    WfiWfe,
    SimdFp,
    IllegalExecutionState,
    Svc(u16),
    Hvc(u16),
    Smc(u16),
    MsrMrsSystem,
    InstructionAbort { kind: Fault, level: u8 },
    PCAlignmentFault,
    DataAbort { kind: Fault, level: u8 },
    SpAlignmentFault,
    TrappedFpu,
    SError,
    Breakpoint,
    Step,
    Watchpoint,
    Brk(u16),
    Other(u32),
}

/// Converts a raw syndrome value (ESR) into a `Syndrome` (ref: D1.10.4).
impl From<u32> for Syndrome {
    fn from(esr: u32) -> Syndrome {
        use self::Syndrome::*;

        let exception_class = (esr >> 26) as u16;
        let is_32_bit_exception = esr & (1 << 25) > 0;
        let instruction_syndrome = esr & ((1 << 25) - 1);

        match exception_class {
            0b00_0000 => Unknown,
            0b00_0001 => WfiWfe,
            0b00_0111 => SimdFp,
            0b00_1110 => IllegalExecutionState,
            0b01_0101 => Svc(instruction_syndrome as u16),
            0b01_0110 => Hvc(instruction_syndrome as u16),
            0b01_0111 => Smc(instruction_syndrome as u16),
            0b01_1000 => MsrMrsSystem,
            0b10_0000..=0b10_0001 => InstructionAbort {
                kind: Fault::from(instruction_syndrome),
                level: (instruction_syndrome as u8) & 0b11
            },
            0b10_0010 => PCAlignmentFault,
            0b10_0100..=0b10_0101 => DataAbort {
                kind: Fault::from(instruction_syndrome),
                level: (instruction_syndrome as u8) & 0b11
            },
            0b10_0110 => SpAlignmentFault,
            0b10_1000..=0b10_1100 => TrappedFpu,
            0b10_1111 => SError,
            0b11_0000..=0b11_0001 => Breakpoint,
            0b11_0010..=0b11_0011 => Step,
            0b11_0100..=0b11_0101 => Watchpoint,
            0b11_1100 => Brk(esr as u16),
            _ => Other(esr),
        }
    }
}
