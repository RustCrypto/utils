# [RustCrypto]: Constant-Time Utilities

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][msrv-image]

Constant-time utility library with selection and equality testing support targeting cryptographic
applications. Supports `const fn` where appropriate. Built on the [`cmov`] crate which provides
architecture-specific predication intrinsics. Heavily inspired by the `subtle` crate.

## About

This crate contains constant-time equivalents of the `bool` and `Option` types (`Choice` and
`CtOption`), along with traits that can be used in combination with them.

The `CtOption` type notably provides eagerly evaluated combinator methods (as opposed to the lazily
evaluated combinators on `Option`) which make it possible to write constant-time code using
an idiomatic Rust style.

This is an experimental next-generation constant-time library inspired by `subtle`, but for now we
recommend you continue to stick with `subtle`. We may attempt to get some of the changes in this
library incorporated into `subtle` for a potential v3.0.

## ⚠️ Security Warning

The implementation contained in this crate has never been independently audited!

USE AT YOUR OWN RISK!

## Minimum Supported Rust Version (MSRV) Policy

MSRV increases are not considered breaking changes and can happen in patch releases.

The crate MSRV accounts for all supported targets and crate feature combinations.

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

[crate-image]: https://img.shields.io/crates/v/ctutils.svg
[crate-link]: https://crates.io/crates/ctutils
[docs-image]: https://docs.rs/ctutils/badge.svg
[docs-link]: https://docs.rs/ctutils/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[msrv-image]: https://img.shields.io/badge/rustc-1.87+-blue.svg
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/ctutils.yml/badge.svg?branch=master
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/ctutils.yml?query=branch:master

[//]: # (links)

[RustCrypto]: https://github.com/RustCrypto
[`cmov`]: https://docs.rs/cmov
