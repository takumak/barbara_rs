# Cortex-M 'Hello world' demo in Rust

## How to run

```
$ rustup override set nightly
$ rustup target add thumbv8m.main-none-eabi
$ rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
$ cargo xtask test
$ cargo xtask run
```
