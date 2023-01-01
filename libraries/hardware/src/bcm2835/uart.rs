use crate::macros::registers::define_registers;

define_registers!(uart_registers, 0x7e21_5000, [
    (aux_irq, u32, 0x00): [
        (MiniUartIrq, 0, Read, bool),
        (SpiOneIrq, 1, Read, bool),
        (SpiTwoIrq, 2, Read, bool),
    ],
    (aux_enable, u32, 0x04): [
        (MiniUartEnable, 0, ReadWrite, bool),
        (SpiOneEnable, 1, ReadWrite, bool),
        (SpiTwoEnable, 2, ReadWrite, bool),
    ],
    (aux_mu_io, u32, 0x40): [
        (Data, 0..7, ReadWrite, u8),
    ],
    (aux_mu_ier, u32, 0x44): [],
    (aux_mu_iir, u32, 0x48): [
        (InterruptPending, 0, Read, bool),
        (InterruptId, 1..2, ReadWrite),
    ],
    (aux_mu_lcr, u32, 0x4c): [
        (DataSize, 0..1, ReadWrite),
        (Break, 6, ReadWrite),
        (DlabAccess, 7, ReadWrite),
    ],
    (aux_mu_mcr, u32, 0x50): [
        (Rts, 1, ReadWrite),
    ],
    (aux_mu_lsr, u32, 0x54): [
        (DataReady, 0, Read, bool),
        (ReceiverOverrun, 1, ReadWrite, bool),
        (TransmitterEmpty, 5, Read, bool),
        (TransmitterIdle, 6, Read, bool),
    ],
    (aux_mu_msr, u32, 0x58): [
        (CtsStatus, 5, Read),
    ],
    (aux_mu_scratch, u32, 0x5c): [
        (Scratch, 0..7, ReadWrite, u8),
    ],
    (aux_mu_cntl, u32, 0x60): [
        (ReceiverEnable, 0, ReadWrite),
        (TransmitterEnable, 1, ReadWrite),
        (EnableReceiveAutoFlowControl, 2, ReadWrite),
        (EnableTransmitAutoFlowControl, 3, ReadWrite),
        (RtsAutoFlowLevel, 4..5, ReadWrite),
        (RtsLevelAssert, 6, ReadWrite),
        (CtsLevelAssert, 7, ReadWrite),
    ],
    (aux_mu_stat, u32, 0x64): [
        (SymbolAvailable, 0, Read),
        (SpaceAvailable, 1, Read),
        (ReceiverIsIdle, 2, Read),
        (TransmitterIsIdle, 3, Read),
        (ReceiverOverrun, 4, Read),
        (TransmitFifoIsFull, 5, Read),
        (RtsStatus, 6, Read),
        (CtsLine, 7, Read),
        (TransmitFifoIsEmpty, 8, Read),
        (TransmitterDone, 9, Read),
        (ReceiveFifoFillLevel, 16..19, Read),
        (TransmitFifoFillLevel, 24..27, Read),
    ],
    (aux_mu_baud, u32, 0x68): [
        (Baudrate, 0..15, ReadWrite, u16),
    ],
]);

pub enum BitOperation {
    Bits7,
    Bits8,
}

pub mod mini_uart {
    use crate::devices::character::CharacterDevice;
    use super::{uart_registers, BitOperation};

    pub struct MiniUartSettings(u64, BitOperation);

    impl MiniUartSettings {
        pub fn new(baud_rate: u64, bit_operation: BitOperation) -> Self {
            Self(baud_rate, bit_operation)
        }
    }

    pub struct MiniUart {
        settings: MiniUartSettings,
    }

    impl MiniUart {
        pub fn new(settings: MiniUartSettings) -> MiniUart {
            let mut mini_uart_enable = uart_registers::aux_enable::MiniUartEnable::new();
            mini_uart_enable.write(true);

            

            Self
        }
    }

    impl CharacterDevice for MiniUart {
        fn read_byte(&mut self) -> shim::io::Result<u8> {
            todo!()
        }

        fn write_byte(&mut self, byte: u8) -> shim::io::Result<()> {
            todo!()
        }

        fn flush(&mut self) -> shim::io::Result<()> {
            Ok(())
        }
    }
}

