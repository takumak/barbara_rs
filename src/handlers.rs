use core::panic::PanicInfo;

use crate::main;

#[no_mangle]
pub extern "C" fn _start_rs() -> ! {
    main()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {};
}

pub union Vector {
    reserved: u32,
    handler: unsafe extern "C" fn(),
}

extern "C" {
    fn __reset();
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
pub static __vector_table: [Vector; 16] = [
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
