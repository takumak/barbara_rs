#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]

mod handlers;
mod backtrace;
mod semihosting;
mod console;
mod arm_uart;

use arm_uart::ArmUart;

const __CONSOLE: *mut ArmUart = 0x4020_0000 as *mut ArmUart;

pub fn main() -> ! {
    console::init();
    println!("=========================================");
    println!("  mps2-an521 'Hello world' DEMO in Rust  ");
    println!("=========================================");

    unsafe {
        asm!("bkpt 0x80")
    }

    semihosting::shutdown();
    loop {};
}
