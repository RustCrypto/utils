# [RustCrypto]: Cryptographic Big Integers

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]
[![Build Status][build-image]][build-link]

Pure Rust implementation of a big integer library which has been designed from
the ground-up for use in cryptographic applications.

Provides constant-time, `no_std`-friendly implementations of modern formulas
using const generics.

[Documentation][docs-link]

# Minimum Supported Rust Version

**Rust 1.51** at a minimum.

## Goals

- No heap allocations (`no_std`-friendly)
- Constant-time by default. We may add variable-time operations in the future
  but they will be secondary and explicitly marked as such.
- Leverage what is possible today with const generics on `stable` rust.
- Support `const fn` as much as possible, including decoding big integers from
  bytes/hex and performing arithmetic operations on them, with the goal of
  being able to compute values at compile-time.

## Status

This library presently provides only a baseline level of functionality.
It's new, unaudited, and may contain bugs. We recommend that it only be
used in an experimental capacity for now.

Please see the [feature wishlist tracking ticket] for more information.

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

[crate-image]: https://img.shields.io/crates/v/crypto-bigint.svg
[crate-link]: https://crates.io/crates/crypto-bigint
[docs-image]: https://docs.rs/crypto-bigint/badge.svg
[docs-link]: https://docs.rs/crypto-bigint/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.51+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/workflows/crypto-bigint/badge.svg?branch=master&event=push
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/crypto-bigint.yml

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
[feature wishlist tracking ticket]: https://github.com/RustCrypto/utils/issues/453
