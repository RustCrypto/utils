# [RustCrypto]: CPU Intrinsics

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][rustc-image]
[![Build Status][build-image]][build-link]

High-level wrappers for architecture-specific CPU intrinsics which are not
yet provided via [`core::arch`], implemented using inline assembly.

[Documentation]

## About

Certain CPU instructions which are important for cryptographic applications
are difficult to emit from LLVM due to various complicating factors.

This crate provides high-level architecture-specific wrappers for these
instructions built on stable inline assembly. No attempts at abstraction
are made, just raw access to specific CPU instructions which are guaranteed
to be emitted verbatim and not optimized away or otherwise rewritten by the
compiler.

## Supported Instructions

### `x86` (32-bit)

- `cmovz` (a.k.a. `cmove`)
- `cmovnz` (a.k.a. `cmovne`)

### `x86_64`

- `cmovz` (a.k.a. `cmove`)
- `cmovnz` (a.k.a. `cmovne`)

## Minimum Supported Rust Version

Rust **1.59** or newer.

In the future, we reserve the right to change MSRV (i.e. MSRV is out-of-scope
for this crate's SemVer guarantees), however when we do it will be accompanied by
a minor version bump.

## License

Licensed under either of:

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/crypto-intrinsics.svg
[crate-link]: https://crates.io/crates/crypto-intrinsics
[docs-image]: https://docs.rs/crypto-intrinsics/badge.svg
[docs-link]: https://docs.rs/crypto-intrinsics/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.51+-blue.svg
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/crypto-intrinsics.yml/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/crypto-intrinsics.yml

[//]: # (general links)

[RustCrypto]: https://github.com/RustCrypto
[Documentation]: https://docs.rs/crypto-intrinsics
[`core::arch`]: https://doc.rust-lang.org/core/arch/index.html
