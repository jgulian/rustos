use crate::macros::registers::define_registers;

define_registers!(uart_registers, 0x3f21_5000, [
    (AuxIrq, u32, 0x00): [
        (mini_uart_irq, 0, Read, {FieldType: bool, DefaultValue: false,}),
        (spi_one_irq, 1, Read, {FieldType: bool, DefaultValue: false,}),
        (spi_two_irq, 2, Read, {FieldType: bool, DefaultValue: false,}),
    ],
    (AuxEnable, u32, 0x04): [
        (mini_uart_enable, 0, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (spi_one_enable, 1, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (spi_two_enable, 2, ReadWrite, {FieldType: bool, DefaultValue: false,}),
    ],
    (AuxMuIo, u32, 0x40): [
        (data, 0..7, ReadWrite, {FieldType: u8, }),
    ],
    (AuxMuIer, u32, 0x44): [],
    (AuxMuIir, u32, 0x48): [
        (interrupt_pending, 0, Read, {FieldType: bool, }),
        (interrupt_id, 1..2, ReadWrite, { CustomType: InterruptId {
            NoInterrupts = 0b00,
            TransmitHoldingRegisterEmpty = 0b01,
            ReceiverHoldsValidByte = 0b10,
        }, }),
    ],
    (AuxMuLcr, u32, 0x4c): [
        (data_size, 0..1, ReadWrite, { CustomType: DataSize {
            Bits7 = 0b00,
            Bits8 = 0b11,
        }, }),
    ],
    (AuxMuLsr, u32, 0x54): [
        (DataReady, 0, Read, {FieldType: bool, DefaultValue: false,}),
        (ReceiverOverrun, 1, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (TransmitterEmpty, 5, Read, {FieldType: bool, DefaultValue: false,}),
        (TransmitterIdle, 6, Read, {FieldType: bool, DefaultValue: false,}),
    ],
    (AuxMuControl, u32, 0x60): [
        (receiver_enable, 0, ReadWrite, {FieldType: bool, DefaultValue: true,}),
        (transmitter_enable, 1, ReadWrite, {FieldType: bool, DefaultValue: true,}),
        (enable_receive_auto_flow_control, 2, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (enable_transmit_auto_flow_control, 3, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (rts_AutoFlowLevel, 4..5, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (rts_level_assert, 6, ReadWrite, {FieldType: bool, DefaultValue: false,}),
        (cts_level_assert, 7, ReadWrite, {FieldType: bool, DefaultValue: false,}),
    ],
    (AuxMuBaud, u32, 0x68): [
        (buadrate, 0..15, ReadWrite, {FieldType: u16,}),
    ],
]);

pub mod mini_uart {
    use crate::devices::character::CharacterDevice;
    use crate::peripheral::character::CharacterDevice;
    use super::{uart_registers, BitOperation};

    pub struct MiniUart;

    impl MiniUart {
        pub fn new() -> MiniUart {
            use uart_registers::{AuxEnable, AuxMuControl, AuxMuLcr, DataSize, AuxMuBaud};

            let mut aux_enable = AuxEnable::default();
            aux_enable.mini_uart_enable = true;
            aux_enable.write();

            let mut mini_uart_control = AuxMuControl::default();
            mini_uart_control.receiver_enable = false;
            mini_uart_control.transmitter_enable = false;
            mini_uart_control.write();

            let mut mini_uart_lcr = AuxMuLcr::default();
            mini_uart_lcr.data_size = DataSize::Bits8;
            mini_uart_lcr.write();

            let mut mini_uart_baud = AuxMuBaud::default();
            mini_uart_baud.buadrate = 270;
            mini_uart_baud.write();

            mini_uart_control = AuxMuControl::default();
            mini_uart_control.receiver_enable = true;
            mini_uart_control.transmitter_enable = true;
            mini_uart_control.write();

            Self
        }
    }

    impl CharacterDevice for MiniUart {
        fn try_read_byte(&mut self) -> shim::io::Result<Option<u8>> {
            if !self.can_read()? {
                Ok(None)
            } else {
                Ok(Some(uart_registers::AuxMuIo::read().data))
            }
        }

        fn try_write_byte(&mut self, byte: u8) -> shim::io::Result<Option<()>> {
            if !self.can_read()? {
                Ok(None)
            } else {
                let mut io = uart_registers::AuxMuIo::default();
                io.data = byte;
                io.write();
                Ok(Some(()))
            }
        }

        fn can_read(&mut self) -> shim::io::Result<bool> {
            Ok(uart_registers::AuxMuLsr::read().DataReady)
        }

        fn can_write(&mut self) -> shim::io::Result<bool> {
            Ok(uart_registers::AuxMuLsr::read().TransmitterEmpty)
        }

        fn flush(&mut self) -> shim::io::Result<()> {
            Ok(())
        }
    }
}

