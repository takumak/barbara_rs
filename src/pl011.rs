use crate::console::Console;

use tock_registers::{
    registers::{ReadOnly, WriteOnly},
    interfaces::{Readable, Writeable},
    register_structs,
    register_bitfields,
};

register_structs! {
    pub PL011 {
        (0x000 => uartdr: WriteOnly<u8>),
        (0x018 => uartfr: ReadOnly<u32, UARTFR::Register>),
        (0x100 => @END),
    }
}

register_bitfields! [
    u32,
    UARTFR [
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
    ],
];

impl Console for PL011 {
    fn putc(&mut self, byte: u8) {
        while self.uartfr.is_set(UARTFR::TXFF) {}
        self.uartdr.set(byte)
    }

    fn flush(&self) {
        while self.uartfr.is_set(UARTFR::BUSY) {}
    }
}
