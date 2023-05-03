# [RustCrypto]: Hybrid Const Generic / Typenum Arrays

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Safety Dance][safety-image]][safety-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Hybrid array type combining const generics with the expressiveness of
[`typenum`]-based constraints, providing an alternative to [`generic-array`]
and a incremental transition path to const generics.

[Documentation][docs-link]

## About

This crate uses `typenum` to enable the following features which aren't yet
possible with the stable implementation of const generics:

- [#60551: Associated constants in traits can not be used in const generics][rust-issue-60551]
- [#76560: Complex generic constants: `feature(generic_const_exprs)`][rust-issue-76560]

Internally the crate is built on const generics and provides traits which make
it possible to convert between const generic types and `typenum` types.

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

[crate-image]: https://buildstats.info/crate/hybrid-array
[crate-link]: https://crates.io/crates/hybrid-array
[docs-image]: https://docs.rs/hybrid-array/badge.svg
[docs-link]: https://docs.rs/hybrid-array/
[build-image]: https://github.com/RustCrypto/utils/workflows/hybrid-array/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/hybrid-array.yml
[safety-image]: https://img.shields.io/badge/unsafe-forbidden-success.svg
[safety-link]: https://github.com/rust-secure-code/safety-dance/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.65+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils

[//]: # (links)

[RustCrypto]: https://github.com/rustcrypto
[RustCrypto/utils#378]: https://github.com/RustCrypto/utils/issues/378
[`typenum`]: https://github.com/paholg/typenum
[`generic-array`]: https://github.com/fizyk20/generic-array
[rust-issue-60551]: https://github.com/rust-lang/rust/issues/60551
[rust-issue-76560]: https://github.com/rust-lang/rust/issues/76560
