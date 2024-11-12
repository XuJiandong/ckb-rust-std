# ckb-rust-std

A collection of `no_std` compatible modules ported from Rust's standard library,
with an initial focus on the `io` module. This crate provides essential
functionality for `no_std` environments while maintaining API compatibility with
`std`. Additional modules may be added in future releases.

## Background

Currently, Rust lacks a standard `no_std` I/O implementation (see
[rust#48331](https://github.com/rust-lang/rust/issues/48331) and
[rfcs#2262](https://github.com/rust-lang/rfcs/issues/2262)). While we await an
official implementation, this crate provides the necessary I/O functionality for
`no_std` environments.

## Features

* Full `no_std` `io` module compatibility for embedded and bare-metal environments
* Compatible with stable Rust (no nightly features required)
* Drop-in replacement for `std::io` with identical semantics and API
* Comprehensive I/O traits and types ported from the standard library

## Rust Error Compatibility
For Rust versions prior to 1.81.0, `core::error::Error` is not available in `no_std` environments. To maintain compatibility:

- For Rust >= 1.81.0: No special configuration needed
- For Rust < 1.81.0: Add the `rust_before_181` feature flag in your `Cargo.toml`:
  ```toml
  [dependencies]
  ckb-core-io = { version = "...", features = ["rust_before_181"] }
  ```
We strongly recommend using Rust 1.81 or later as it provides better error handling features.

## Frequently Asked Questions

### What version of `std::io` is this ported from?
This crate is ported from Rust 1.81's `std::io` implementation.

### Where is the API documentation?
Since this is a direct port of `std::io`, we refer users to the official [Rust
std::io documentation](https://doc.rust-lang.org/std/io/index.html). All traits,
types, and functions maintain identical behavior and semantics to their `std`
counterparts.

### How to adopt official `core::io` if it is implemented?
When an official `core::io` implementation becomes available, migration should be straightforward:

1. Due to identical behavior and semantics, you can simply replace imports from `ckb-core-io` with `core::io`
2. Update your `Cargo.toml` dependencies to remove this crate
3. No behavioral changes will be required in your code

For example:

```rust,ignore
// Before
use ckb_core_io::{Read, Write};

// After
use core::io::{Read, Write};
```

### What features are removed compared to std::io?
Due to the lack of
[specialization](https://github.com/rust-lang/rust/issues/31844) in stable Rust,
the following optimizations and features have been removed:

1. Optimized implementations of `copy()` for specific types
2. Optimized `SizeHint` implementations for certain iterators

The `read_vectored()` and related vectored I/O operations is removed due to
limited use cases in no_std environments.

The `repr` of error code is removed.