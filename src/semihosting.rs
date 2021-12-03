pub fn shutdown() {
    unsafe {
        asm!(
            "bkpt 0xab",
            in("r0") 0x18,
            in("r1") 0x20026,
        )
    }
}
