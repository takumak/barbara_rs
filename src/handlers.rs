use core::{
    panic::PanicInfo,
    ptr,
};

#[no_mangle]
#[naked]
pub unsafe extern "C" fn __reset() {
    asm!(
        // init sp
        "ldr r0, =__stack_bottom",
        "mov sp, r0",

        // disable irq
        "cpsid i",

        // goto Rust function
        "ldr r0, =__init_runtime",
        "bx r0",
        options(noreturn),
    )
}

#[no_mangle]
fn __init_runtime() {
    extern "C" {
        static mut __bss_start: u8;
        static mut __bss_end: u8;
        static mut __data_start: u8;
        static mut __data_end: u8;
        static __rodata_start: u8;
    }

    unsafe {
        let size =
            &__bss_end as *const u8 as usize -
            &__bss_start as *const u8 as usize;
        ptr::write_bytes(&mut __bss_start as *mut u8, 0, size);

        let size =
            &__data_end as *const u8 as usize -
            &__data_start as *const u8 as usize;
        ptr::copy_nonoverlapping(&__rodata_start as *const u8,
                                 &mut __data_start as *mut u8,
                                 size);
    }

    use crate::main;
    main()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {};
}

union Vector {
    reserved: u32,
    handler: unsafe extern "C" fn(),
}

extern "C" {
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
    Vector { reserved: 0 }, // sp limit
    Vector { handler: __reset },
    Vector { handler: __nmi },
    Vector { handler: __hardfault },
    Vector { handler: __memmanage },
    Vector { handler: __busfault },
    Vector { handler: __usagefault },
    Vector { handler: __securefault },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { handler: __svc },
    Vector { handler: __debugmon },
    Vector { reserved: 0 },
    Vector { handler: __pendsv },
    Vector { handler: __systick },
];

#[no_mangle]
pub extern "C" fn DefaultExceptionHandler() {
    loop {}
}
