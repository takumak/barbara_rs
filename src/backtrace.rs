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

use crate::decl_c_symbol_addr;
decl_c_symbol_addr!(__text_s, text_s);
decl_c_symbol_addr!(__text_e, text_e);
decl_c_symbol_addr!(__stack_s, stack_s);
decl_c_symbol_addr!(__stack_e, stack_e);

#[derive(Clone, Copy)]
struct StackFrame {
    fp: usize,
    lr: usize,
}

pub fn unwind_walk(pc: usize, fp: usize, limit: u32, func: fn(usize)) {
    let mut fp_ = fp;

    func(pc);

    for _i in 1..limit {
        if fp_ < stack_s() || fp_ >= stack_e() {
            break;
        }

        let frame: StackFrame = unsafe { *(fp_ as *const StackFrame) };
        if frame.lr < text_s() || frame.lr >= text_e() {
            break;
        }

        func(frame.lr);
        fp_ = frame.fp;
    }
}

#[allow(dead_code)]
pub fn trace(limit: u32, func: fn(usize)) {
    unsafe {
        let fp: usize;
        asm!("mov {}, r7", out(reg) fp);
        let ref frame: StackFrame = *(fp as *const StackFrame);
        unwind_walk(frame.lr, frame.fp, limit, func);
    }
}
