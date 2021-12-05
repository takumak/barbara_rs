/*

On armv8m (T32 ISA), frame pointer is stored in R7
(R11 for A32, X29 for AArch64)

function:
    push {r7, lr}
    // function body
    pop  {r7, pc}

[refs]
- https://developer.arm.com/documentation/100067/0607/armclang-Command-line-Options/-fomit-frame-pointer---fno-omit-frame-pointer
- linux/arch/arm64/kernel/stacktrace.c

*/

struct StackFrame {
    fp: usize,
    lr: usize,
}

pub fn unwind_walk(pc: usize, fp: usize, limit: u32, func: fn(usize)) {
    unsafe {
        extern "C" {
            static __text_s: u8;
            static __text_e: u8;
            static __stack_s: u8;
            static __stack_e: u8;
        }

        let text_s = &__text_s as *const u8 as usize;
        let text_e = &__text_e as *const u8 as usize;
        let stack_s = &__stack_s as *const u8 as usize;
        let stack_e = &__stack_e as *const u8 as usize;

        let mut fp_ = fp;

        func(pc);

        for _i in 1..limit {
            if fp_ < stack_s || fp_ >= stack_e {
                break;
            }

            let ref frame: StackFrame = *(fp_ as *const StackFrame);
            if frame.lr < text_s || frame.lr >= text_e {
                break;
            }

            func(frame.lr);
            fp_ = frame.fp;
        }
    }
}
