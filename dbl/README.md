# [RustCrypto]: Double operation in `GF(2^n)`

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Double operation (a.k.a. "multiply-by-x") in Galois Field `GF(2^n)` using
the lexicographically first polynomial among the irreducible degree `n` polynomials
having a minimum number of coefficients.

## Minimum Supported Rust Version (MSRV) Policy

MSRV increases are not considered breaking changes and can happen in patch
releases.

The crate MSRV accounts for all supported targets and crate feature
combinations, excluding explicitly unstable features.

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/dbl.svg
[crate-link]: https://crates.io/crates/dbl
[docs-image]: https://docs.rs/dbl/badge.svg
[docs-link]: https://docs.rs/dbl/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/dbl.yml/badge.svg?branch=master
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/dbl.yml?query=branch:master

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
