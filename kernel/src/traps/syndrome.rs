use core::fmt;
use core::fmt::Formatter;

use crate::traps::syndrome::Fault::{AccessFlag, AddressSize, Alignment, Permission, TlbConflict, Translation};

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
pub enum FaultStatusCode {
    AddressSizeFault0,
    AddressSizeFault1,
    AddressSizeFault2,
    AddressSizeFault3,
    TranslationFault0,
    TranslationFault1,
    TranslationFault2,
    TranslationFault3,
    AccessFlagFault1,
    AccessFlagFault2,
    AccessFlagFault3,
    PermissionFault1,
    PermissionFault2,
    PermissionFault3,
    SynchronousExternalAbortNotOnTranslationWalk,
    SynchronousParityOrECCNotOnTranslationWalk,
    SynchronousExternalAbortOnTranslationWalk0,
    SynchronousExternalAbortOnTranslationWalk1,
    SynchronousExternalAbortOnTranslationWalk2,
    SynchronousExternalAbortOnTranslationWalk3,
    SynchronousParityOrECCOnTranslationWalk0,
    SynchronousParityOrECCOnTranslationWalk1,
    SynchronousParityOrECCOnTranslationWalk2,
    SynchronousParityOrECCOnTranslationWalk3,
    AlignmentFault,
    TlbConflictAbort,
    UnsupportedAtomicHardwareUpdate,
    LockdownFault,
    UnsupportedExclusiveOrAtomicAccess,
    SectionDomainFault,
    PageDomainFault,
    Unknown(u32)
}

impl From<u32> for FaultStatusCode {
    fn from(value: u32) -> Self {
        match value & 0b111111 {
            0b000000 => FaultStatusCode::AddressSizeFault0,
            0b000001 => FaultStatusCode::AddressSizeFault1,
            0b000010 => FaultStatusCode::AddressSizeFault2,
            0b000011 => FaultStatusCode::AddressSizeFault3,
            0b000100 => FaultStatusCode::TranslationFault0,
            0b000101 => FaultStatusCode::TranslationFault1,
            0b000110 => FaultStatusCode::TranslationFault2,
            0b000111 => FaultStatusCode::TranslationFault3,
            0b001001 => FaultStatusCode::AccessFlagFault1,
            0b001010 => FaultStatusCode::AccessFlagFault2,
            0b001011 => FaultStatusCode::AccessFlagFault3,
            0b001101 => FaultStatusCode::PermissionFault1,
            0b001110 => FaultStatusCode::PermissionFault2,
            0b001111 => FaultStatusCode::PermissionFault3,
            0b010000 => FaultStatusCode::SynchronousExternalAbortNotOnTranslationWalk,
            0b011000 => FaultStatusCode::SynchronousParityOrECCNotOnTranslationWalk,
            0b010100 => FaultStatusCode::SynchronousExternalAbortOnTranslationWalk0,
            0b010101 => FaultStatusCode::SynchronousExternalAbortOnTranslationWalk1,
            0b010110 => FaultStatusCode::SynchronousExternalAbortOnTranslationWalk2,
            0b010111 => FaultStatusCode::SynchronousExternalAbortOnTranslationWalk3,
            0b011100 => FaultStatusCode::SynchronousParityOrECCOnTranslationWalk0,
            0b011101 => FaultStatusCode::SynchronousParityOrECCOnTranslationWalk1,
            0b011110 => FaultStatusCode::SynchronousParityOrECCOnTranslationWalk2,
            0b011111 => FaultStatusCode::SynchronousParityOrECCOnTranslationWalk3,
            0b100001 => FaultStatusCode::AlignmentFault,
            0b110000 => FaultStatusCode::TlbConflictAbort,
            0b110001 => FaultStatusCode::UnsupportedAtomicHardwareUpdate,
            0b110100 => FaultStatusCode::LockdownFault,
            0b110101 => FaultStatusCode::UnsupportedExclusiveOrAtomicAccess,
            0b111101 => FaultStatusCode::SectionDomainFault,
            0b111110 => FaultStatusCode::PageDomainFault,
            v => FaultStatusCode::Unknown(v)
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct AbortData {
    pub write: bool,
    pub fault_status_code: FaultStatusCode,
}

impl From<u32> for AbortData {
    fn from(value: u32) -> Self {
        AbortData {
            write: (value & (0b1 << 6)) > 0,
            fault_status_code: FaultStatusCode::from(value),
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
    InstructionAbort(AbortData),
    PCAlignmentFault,
    DataAbort(AbortData),
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
            0b10_0000..=0b10_0001 => InstructionAbort(AbortData::from(instruction_syndrome)),
            0b10_0010 => PCAlignmentFault,
            0b10_0100..=0b10_0101 => DataAbort(AbortData::from(instruction_syndrome)),
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

impl fmt::Display for Syndrome {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Syndrome::Unknown => f.write_str("Unknown"),
            Syndrome::WfiWfe => f.write_str("WfiWfe"),
            Syndrome::SimdFp => f.write_str("SimdFp"),
            Syndrome::IllegalExecutionState => f.write_str("IllegalExecutionState"),
            Syndrome::Svc(data) =>
                f.debug_struct("Svc")
                    .field("data", data)
                    .finish(),
            Syndrome::Hvc(data) =>
                f.debug_struct("Hvc")
                    .field("data", data)
                    .finish(),
            Syndrome::Smc(data) =>
                f.debug_struct("Smc")
                    .field("data", data)
                    .finish(),
            Syndrome::MsrMrsSystem => f.write_str("MsrMrsSystem"),
            Syndrome::InstructionAbort(AbortData{ write, fault_status_code }) =>
                f.debug_struct("InstructionAbort")
                    .field("write", write)
                    .field("fault_status_code", fault_status_code)
                    .finish(),
            Syndrome::PCAlignmentFault => f.write_str("PCAlignmentFault"),
            Syndrome::DataAbort(AbortData{ write, fault_status_code }) =>
                f.debug_struct("DataAbort")
                    .field("write", write)
                    .field("fault_status_code", fault_status_code)
                    .finish(),
            Syndrome::SpAlignmentFault => f.write_str("SpAlignmentFault"),
            Syndrome::TrappedFpu => f.write_str("TrappedFpu"),
            Syndrome::SError => f.write_str("SError"),
            Syndrome::Breakpoint => f.write_str("Breakpoint"),
            Syndrome::Step => f.write_str("Step"),
            Syndrome::Watchpoint => f.write_str("Watchpoint"),
            Syndrome::Brk(data) =>
                f.debug_struct("Brk")
                    .field("data", data)
                    .finish(),
            Syndrome::Other(data) =>
                f.debug_struct("Other")
                    .field("data", data)
                    .finish(),
        }
    }
}
