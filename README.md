# mps2-an521 'Hello world' DEMO in Rust

## How to run

```
$ rustup override set nightly
$ rustup target add thumbv8m.main-none-eabi
$ rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.03s
     Running `qemu-system-arm -M mps2-an521 -semihosting -serial stdio -display none -kernel target/thumbv8m.main-none-eabi/debug/mps2_an521_rs`
=========================================
  mps2-an521 'Hello world' DEMO in Rust  
=========================================
==== KERNEL PANIC ====
Unhandled exception: ipsr=00000003
pc : 380006fe  lr : 38008547
sp : 3805fef8  r12: 00000000
r11: deadbeef  r10: 00000000
r9 : 00000000  r8 : 00000000
r7 : 3805ffe0  r6 : 00000000
r5 : 00000000  r4 : 00000000
r3 : 3800059f  r2 : 0000000a
r1 : 00000000  r0 : 00000000
pstate : 61000000

Backtrace:
  380006fe  mps2_an521_rs::main::ha4a67f4ddce969a4 +0x13f
  38000c21  __reset +0xa0
```
