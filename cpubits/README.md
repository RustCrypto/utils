# [RustCrypto]: CPU bits selection

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Build-time selection for 32-bit vs 64-bit backends, with CPU-specific overrides
for producing code with optimal performance.

[Documentation][docs-link]

## About

Many algorithms support 32-bit and 64-bit backends, implemented using
(often arrays of) `u32` or `u64`, in order to better support 32-bit or 64-bit
targets.

Selecting the optimal backend for a given target, however, is not quite as
straightforward as introspecting `target_pointer_width` and choosing the
backend that matches. Many 32-bit targets actually have better performance when
using a backend based on `u64`, even if that isn't the CPU's native word size.

Backend selection often has cross-cutting concerns across crates, which might
need to agree on a common types for representing e.g. big integers as arrays
of "limbs". This crate provides a single, common selection for the word size
for a given target, with configurable overrides which also apply uniformly to
all crates in a given project.

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

[crate-image]: https://img.shields.io/crates/v/cpubits.svg
[crate-link]: https://crates.io/crates/cpubits
[docs-image]: https://docs.rs/cpubits/badge.svg
[docs-link]: https://docs.rs/cpubits/
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/cpubits.yml/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/cpubits.yml
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.56+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
