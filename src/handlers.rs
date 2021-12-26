use core::{arch::asm, panic::PanicInfo, ptr};

#[macro_export]
macro_rules! decl_c_symbol_addr {
    ($sym_name: ident, $wrapper_name: ident) => {
        extern "C" {
            static $sym_name: u8;
        }
        #[inline]
        fn $wrapper_name() -> usize {
            unsafe { &$sym_name as *const _ as usize }
        }
    };
}

decl_c_symbol_addr!(__bss_s, bss_s);
decl_c_symbol_addr!(__bss_e, bss_e);
decl_c_symbol_addr!(__data_s, data_s);
decl_c_symbol_addr!(__data_e, data_e);
decl_c_symbol_addr!(__rodata_s, rodata_s);

#[no_mangle]
unsafe extern "C" fn __reset() {
    // disable interrupt
    asm!("cpsid i");

    let size = bss_e() - bss_s();
    ptr::write_bytes(bss_s() as *mut u8, 0, size);

    let size = data_e() - data_s();
    ptr::copy_nonoverlapping(rodata_s() as *const u8, data_s() as *mut u8, size);

    use crate::main;
    main()
}

#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    use crate::println;
    if let Some(message) = panic_info.message() {
        println!("{}", *message);
    }
    if let Some(location) = panic_info.location() {
        println!("location: {}:{}", location.file(), location.line(),);
    }

    unsafe { asm!("bkpt 0x80") }
    loop {}
}

union Vector {
    reserved: u32,
    handler: unsafe extern "C" fn(),
}

extern "C" {
    fn __stack_e();
    fn __nmi();
    fn __hardfault();
    fn __memmanage();
    fn __busfault();
    fn __usagefault();
    fn __securefault();
    fn __svc();
    fn __debugmon();
    fn __pendsv();
    fn __systick();
}

#[no_mangle]
#[link_section = ".vector_table"]
static __vector_table: [Vector; 16] = [
    Vector { handler: __stack_e }, // initial sp
    Vector { handler: __reset },
    Vector { handler: __nmi },
    Vector {
        handler: __hardfault,
    },
    Vector {
        handler: __memmanage,
    },
    Vector {
        handler: __busfault,
    },
    Vector {
        handler: __usagefault,
    },
    Vector {
        handler: __securefault,
    },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: __svc },
    Vector {
        handler: __debugmon,
    },
    Vector { reserved: 0 },
    Vector { handler: __pendsv },
    Vector { handler: __systick },
];

struct ExceptionRegs {
    r13: u32,
    r4: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
    r9: u32,
    r10: u32,
    r11: u32,
    r0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
    r12: u32,
    r14: u32,
    return_address: u32,
    pstate: u32,
}

#[no_mangle]
#[naked]
unsafe extern "C" fn DefaultExceptionHandler() {
    /*

    +----------------+  <- Next SP
    |    R13 (SP)    |
    |       R4       |
    |       R5       |
    |       R6       |
    |       R7       |
    |       R8       |
    |       R9       |
    |       R10      |
    |       R11      |
    +----------------+  <- Current SP
    |       R0       |
    |       R1       |
    |       R2       |
    |       R3       |
    |       R12      |
    |    R14 (LR)    |
    | Return address |
    |      xPSR      |
    +----------------+  <- Previous SP
    |                |
    +----------------+

     */

    asm!(
        // calculate previous SP -> r3
        "mov r3, sp",
        "add r3, r3, #32",
        "stmfd sp!, {{r3-r11}}",
        "mov r0, sp",
        "b __unhandled_exception",
        options(noreturn)
    )
}

#[no_mangle]
unsafe extern "C" fn __unhandled_exception(regs_addr: usize) {
    let ref regs: ExceptionRegs = *(regs_addr as *const ExceptionRegs);

    let ipsr: u32;

    asm!(
        "mrs {0}, ipsr",
        out(reg) ipsr,
    );

    use crate::println;

    println!("==== KERNEL PANIC ====");
    println!("Unhandled exception: ipsr={:08x}", ipsr);
    println!("pc : {:08x}  lr : {:08x}", regs.return_address, regs.r14);
    println!("sp : {:08x}  r12: {:08x}", regs.r13, regs.r12);
    println!("r11: {:08x}  r10: {:08x}", regs.r11, regs.r10);
    println!("r9 : {:08x}  r8 : {:08x}", regs.r9, regs.r8);
    println!("r7 : {:08x}  r6 : {:08x}", regs.r7, regs.r6);
    println!("r5 : {:08x}  r4 : {:08x}", regs.r5, regs.r4);
    println!("r3 : {:08x}  r2 : {:08x}", regs.r3, regs.r2);
    println!("r1 : {:08x}  r0 : {:08x}", regs.r1, regs.r0);
    println!("pstate : {:08x}", regs.pstate);

    println!();
    println!("Backtrace:");
    use crate::backtrace;
    backtrace::unwind_walk(
        regs.return_address as usize,
        regs.r7 as usize,
        10,
        |addr: usize| {
            use crate::kallsyms;
            let mut buf: [u8; 128] = [0; 128];
            match kallsyms::safe_search(addr, &mut buf) {
                Some((name, off)) => println!("  {:08x}  {} +{:#x}", addr, name, off),
                None => println!("  {:08x}", addr),
            }
        },
    );

    use crate::semihosting;
    semihosting::shutdown();

    loop {}
}
