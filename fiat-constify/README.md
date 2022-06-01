# [RustCrypto]: `const fn` postprocessor for `fiat-crypto`

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Postprocessor for [fiat-crypto] generated field implementations which rewrites
them as `const fn`.

[Documentation][docs-link]

## About

This crate is a workaround for [mit-plv/fiat-crypto#1086] which provides a
mechanical postprocessing step for [fiat-crypto] generated field
implementations.

It removes `&mut` references which aren't yet stable with `const fn`, instead
allocating and returning an array on the stack for results.

## License

Licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://buildstats.info/crate/fiat-constify
[crate-link]: https://crates.io/crates/fiat-constify
[docs-image]: https://docs.rs/fiat-constify/badge.svg
[docs-link]: https://docs.rs/fiat-constify/
[build-image]: https://github.com/RustCrypto/utils/workflows/fiat-constify/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/fiat-constify.yml
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.56+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils

[//]: # (links)

[RustCrypto]: https://github.com/rustcrypto
[fiat-crypto]: https://github.com/mit-plv/fiat-crypto/
[mit-plv/fiat-crypto#1086]: https://github.com/mit-plv/fiat-crypto/issues/1086
