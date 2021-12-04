# mps2-an521 'Hello world' DEMO in Rust

## How to run

```
$ cargo run
    Updating git repository `https://github.com/tock/tock`
   Compiling mps2_an521_rs v0.1.0 (/home/kawai/asos)
   Compiling tock-registers v0.7.0 (https://github.com/tock/tock?rev=b8f5da9c09373a5bbf48e70e9cef16d669f3cdde#b8f5da9c)
    Finished dev [unoptimized + debuginfo] target(s) in 15.45s
     Running `qemu-system-arm -M mps2-an521 -semihosting -serial stdio -display none -kernel target/thumbv8m.main-none-eabi/debug/mps2_an521_rs`
=====================================
mps2-an521 'Hello world' DEMO in Rust
=====================================
```
