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

`core::error::Error` has been stable since Rust 1.81.0, so this crate uses it
directly and requires **Rust 1.81.0 or later**. No special configuration is
needed; the crate works out of the box with any compatible toolchain. We
strongly recommend using the latest stable Rust (e.g. 1.97.1) for the best
experience.

## Frequently Asked Questions

### What version of `std::io` is this ported from?
This crate is ported from Rust 1.97.1's `std::io` implementation.

### Where is the API documentation?
Since this is a direct port of `std::io`, we refer users to the official [Rust
std::io documentation](https://doc.rust-lang.org/std/io/index.html). All traits,
types, and functions maintain identical behavior and semantics to their `std`
counterparts.

### How to adopt official `core::io` if it is implemented?
When an official `core::io` implementation becomes available, migration should be straightforward:

1. Due to identical behavior and semantics, you can simply replace imports from `ckb-rust-std::io` with `core::io`
2. Update your `Cargo.toml` dependencies to remove this crate
3. No behavioral changes will be required in your code

For example:

```rust,ignore
// Before
use ckb_rust_std::io::{Read, Write};

// After
use core::io::{Read, Write};
```

### What features are removed compared to std::io?
- Optimized implementations of `copy()` for specific types, due to the lack of
  [specialization](https://github.com/rust-lang/rust/issues/31844) in stable Rust
- Optimized `SizeHint` implementations for certain iterators, due to the lack of
  [specialization](https://github.com/rust-lang/rust/issues/31844) in stable Rust
- Vectored I/O operations (`read_vectored()` and related functions) due to limited
  use cases in `no_std` environments
- Error code representations (`repr`)
- The `stdio` module
