extern crate bitfield;
extern crate mmio;

use bitfield::bitfield;

use mmio::{Readable, RegisterR, RegisterRW, Writeable};

bitfield! {
    State: u32 {
        TX_BF[0];
        RX_BF[1];
    }
}

bitfield! {
    Ctrl: u32 {
        TX_EN[0];
        RX_EN[1];
        TX_INTR_EN[2];
        RX_INTR_EN[3];
    }
}

pub struct ArmUart {
    data: RegisterRW<0x000, u8, u8>,
    state: RegisterR<0x004, u32, State>,
    ctrl: RegisterRW<0x008, u32, Ctrl>,
}

use crate::console::Console;

impl Console for ArmUart {
    fn init(&mut self) {
        self.ctrl.write(Ctrl::TX_EN)
    }

    fn putc(&mut self, byte: u8) {
        while self.state.read().is_set(State::TX_BF) {}
        self.data.write(byte)
    }

    fn flush(&self) {
        while self.state.read().is_set(State::TX_BF) {}
    }
}
