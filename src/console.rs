use core::fmt;
use core::fmt::Write;

pub trait Console {
    fn init(&mut self) { }
    fn putc(&mut self, byte: u8);
    fn flush(&self);
}

struct Writer {
    console: *mut dyn Console
}

impl Writer {
    fn new(console: *mut dyn Console) -> Self {
        Writer { console }
    }

    fn putc(&mut self, byte: u8) {
        unsafe {
            (*self.console).putc(byte)
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.putc(b'\r')
            }
            self.putc(byte)
        }
        Ok(())
    }
}

pub fn init() {
    use crate::__CONSOLE;
    unsafe {
        (*__CONSOLE).init();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print_fmt(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    use crate::__CONSOLE;
    let mut writer = Writer::new(__CONSOLE);
    let _ = writer.write_fmt(args);
}
