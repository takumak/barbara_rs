use crate::console::Console;

use tock_registers::{
    registers::ReadWrite,
    interfaces::{Readable, Writeable},
    register_structs,
    register_bitfields,
};

register_structs! {
    pub ArmUart {
        (0x000 => data:  ReadWrite<u8>),
        (0x004 => state: ReadWrite<u32, State::Register>),
        (0x008 => ctrl:  ReadWrite<u32, Control::Register>),
        (0x100 => @END),
    }
}

register_bitfields! [
    u32,
    State [
        TX_BF OFFSET(0) NUMBITS(1) [],
        RX_BF OFFSET(1) NUMBITS(1) [],
    ],
    Control [
        TX_EN OFFSET(0) NUMBITS(1) [],
        RX_EN OFFSET(1) NUMBITS(1) [],
        TX_INTR_EN OFFSET(2) NUMBITS(1) [],
        RX_INTR_EN OFFSET(3) NUMBITS(1) [],
    ],
];

impl Console for ArmUart {
    fn init(&mut self) {
        self.ctrl.write(Control::TX_EN.val(1))
    }

    fn putc(&mut self, byte: u8) {
        while self.state.is_set(State::TX_BF) {}
        self.data.set(byte)
    }

    fn flush(&self) {
        while self.state.is_set(State::TX_BF) {}
    }
}
