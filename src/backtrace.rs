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

pub fn backtrace(pc: usize, fp: usize, limit: u32, func: fn(usize)) {
    unsafe {
        extern "C" {
            static __stack_top: u8;
            static __stack_bottom: u8;
        }

        let top = &__stack_top as *const u8 as usize;
        let bottom = &__stack_bottom as *const u8 as usize;
        let mut fp_ = fp;

        func(pc);

        for _i in 1..limit {
            if fp_ < top || fp_ >= bottom {
                break;
            }

            let ref frame: StackFrame = *(fp_ as *const StackFrame);
            if frame.lr & 0xf000_0000 == 0xf000_0000 {
                break;
            }

            func(frame.lr);
            fp_ = frame.fp;
        }
    }
}
