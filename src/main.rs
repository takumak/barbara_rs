#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(linkage)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

mod arm_uart;
mod backtrace;
mod console;
mod handlers;
mod heap;
mod kallsyms;
mod semihosting;

use arm_uart::ArmUart;
const __CONSOLE: *mut ArmUart = 0x4020_0000 as *mut ArmUart;

use core::arch::asm;

pub fn main() -> ! {
    console::init();
    println!("=========================================");
    println!("   Cortex-M 'Hello world' demo in Rust   ");
    println!("=========================================");

    heap::init();

    use alloc::vec::Vec;
    let mut v = Vec::new();
    for i in 0..10 {
        v.push(i);
    }
    println!("vector: {:?}", v);

    println!();
    println!("make panic");

    unsafe {
        asm!(
            "mov r11, #0xbeef",
            "movt r11, #0xdead",
            "bkpt 0x80",
            out("r11") _
        )
    }

    semihosting::shutdown();
    loop {}
}
