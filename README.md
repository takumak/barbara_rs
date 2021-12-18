# Cortex-M 'Hello world' demo in Rust

## How to run

```
$ rustup override set nightly
$ rustup target add thumbv8m.main-none-eabi
$ cargo xtask run
```

## Test

```
$ cargo test --workspace --lib
```

## Test and Coverage measuring

```
$ rustup component add llvm-tools-preview
$ cargo install cargo-binutils
$ cargo install rustfilt
$ cargo xtask testall
```
