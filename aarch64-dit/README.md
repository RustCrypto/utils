# [RustCrypto]: AArch64 Data-Independent Timing (DIT)

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][msrv-image]

Wrappers for enabling/disabling the [Data-Independent Timing] feature of modern AArch64 CPUs which
can be used to help ensure that instructions take a constant amount of time regardless of input
data, thus preventing potential information leaks via timing sidechannels.

[Documentation][docs-link]

## Minimum Supported Rust Version

Rust **1.85** or newer.

In the future, we reserve the right to change MSRV (i.e. MSRV is out-of-scope for this crate's
SemVer guarantees), however when we do it will be accompanied by a minor version bump.

## License

Licensed under either of:

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/aarch64-dit.svg
[crate-link]: https://crates.io/crates/aarch64-dit
[docs-image]: https://docs.rs/aarch64-dit/badge.svg
[docs-link]: https://docs.rs/aarch64-dit/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[msrv-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/aarch64-dit.yml/badge.svg?branch=master
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/aarch64-dit.yml?query=branch:master

[//]: # (links)

[RustCrypto]: https://github.com/RustCrypto
[Data-Independent Timing]: https://developer.arm.com/documentation/ddi0595/2021-06/AArch64-Registers/DIT--Data-Independent-Timing
