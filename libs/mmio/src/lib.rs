#![cfg_attr(not(test), no_std)]

pub trait Readable<const OFF: usize, T, BF>
where
    BF: From<T>,
{
    fn read(&self) -> BF {
        let ptr = (self as *const _ as *const u8 as usize + OFF) as *const T;
        unsafe { ptr.read_volatile() }.into()
    }
}

pub trait Writeable<const OFF: usize, T, BF>
where
    T: From<BF>,
{
    fn write(&mut self, val: BF) {
        let ptr = (self as *mut _ as *mut u8 as usize + OFF) as *mut T;
        let v: T = val.into();
        unsafe { ptr.write_volatile(v) }
    }
}

pub struct RegisterRW<const OFF: usize, T, BF> {
    _t: core::marker::PhantomData<T>,
    _bf: core::marker::PhantomData<BF>,
}
impl<const OFF: usize, T, BF: From<T>> Readable<OFF, T, BF> for RegisterRW<OFF, T, BF> {}
impl<const OFF: usize, T: From<BF>, BF> Writeable<OFF, T, BF> for RegisterRW<OFF, T, BF> {}

pub struct RegisterR<const OFF: usize, T, BF> {
    _t: core::marker::PhantomData<T>,
    _bf: core::marker::PhantomData<BF>,
}
impl<const OFF: usize, T, BF: From<T>> Readable<OFF, T, BF> for RegisterR<OFF, T, BF> {}

pub struct RegisterW<const OFF: usize, T, BF> {
    _t: core::marker::PhantomData<T>,
    _bf: core::marker::PhantomData<BF>,
}
impl<const OFF: usize, T: From<BF>, BF> Writeable<OFF, T, BF> for RegisterW<OFF, T, BF> {}

#[cfg(test)]
mod tests {
    extern crate bitfield;

    use bitfield::bitfield;

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

    use crate::{Readable, RegisterR, RegisterRW, Writeable};

    struct ArmUart {
        data: RegisterRW<0x000, u8, u8>,
        state: RegisterR<0x004, u32, State>,
        ctrl: RegisterRW<0x008, u32, Ctrl>,
    }

    #[test]
    fn test_write() {
        let buf: [u32; 3] = [0, 2, 0];
        let uart = unsafe { &mut *(buf.as_ptr() as usize as *mut ArmUart) };

        uart.data.write(0xaa);
        uart.ctrl
            .write(uart.ctrl.read() | Ctrl::TX_EN | Ctrl::RX_EN);

        assert_eq!(buf, [0xaa, 2, 3]);
    }

    #[test]
    fn test_read() {
        let buf: [u32; 3] = [0xbb, 2, 5];
        let uart = unsafe { &mut *(buf.as_ptr() as usize as *mut ArmUart) };

        assert_eq!(uart.data.read(), 0xbb);
        assert_eq!(uart.state.read(), State::RX_BF);
        assert_eq!(uart.ctrl.read(), Ctrl::TX_EN | Ctrl::TX_INTR_EN);
    }
}
