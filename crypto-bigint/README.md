# [RustCrypto]: Cryptographic Big Integers

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]
[![Build Status][build-image]][build-link]

Pure Rust implementation of a big integer library designed from the ground-up
for use in cryptographic applications only. Provides constant-time,
no_std-friendly implementations of modern formulas using const generics.

[Documentation][docs-link]

## Status

tl;dr: not ready to use.

This is a work-in-progress prototype of what is possible in a const generics-based
big integer library oriented towards cryptography.

However, const generics support in Rust is still in a very early stage and thus
so is this library. We intend to evolve it along with Rust's support for const
generics until they meet our baseline requirements.

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
