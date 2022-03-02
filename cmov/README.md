# [RustCrypto]: Conditional Move Intrinsics

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][msrv-image]

Conditional move CPU intrinsics which are guaranteed to execute in
constant-time and not be rewritten as branches by the compiler.

Provides wrappers for the [CMOV family] of instructions on x86/x86_64 and
the [CSEL] instruction on AArch64 CPUs.

[Documentation][docs-link]

## About

Conditional move intrinsics provide [predication] which allows selection
of one or more values without using branch instructions, thus making the
selection constant-time with respect to the values, and not subject to CPU
execution features which might introduce timing or other microarchitectural
sidechannels introduced by branch prediction or other speculative execution
features.

Intel has confirmed that all extant CPUs implement the CMOV family of
instructions in constant-time, and that this property will hold for future
Intel CPUs as well.

This crate provides wrappers for the CMOV/CSEL instructions implemented using
inline assembly as stabilized in Rust 1.59. This means the implementation
is a black box that will not be rewritten by e.g. LLVM's architecture-specific
lowerings, such as the [x86-cmov-conversion] pass.

## Supported target architectures

This crate provides guaranteed constant-time operation using inline assembly
on the following CPU architectures:

- [x] `x86` (`CMOVZ`, `CMOVNZ`)
- [x] `x86_64` (`CMOVZ`, `CMOVNZ`)
- [x] `aarch64` (`CSEL`)

On other target architectures, a "best effort" portable fallback implementation
based on bitwise arithmetic is used instead. However, we cannot guarantee that
this implementation generates branch-free code.

It's possible to extend constant-time guarantees to other CPU  architectures.
Please open an issue with your desired CPU architecture if this interests you.

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

[crate-image]: https://img.shields.io/crates/v/cmov.svg
[crate-link]: https://crates.io/crates/cmov
[docs-image]: https://docs.rs/cmov/badge.svg
[docs-link]: https://docs.rs/cmov/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[msrv-image]: https://img.shields.io/badge/rustc-1.59+-blue.svg
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/cmov.yml/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/cmov.yml

[//]: # (general links)

[RustCrypto]: https://github.com/RustCrypto
[CMOV family]: https://www.jaist.ac.jp/iscenter-new/mpc/altix/altixdata/opt/intel/vtune/doc/users_guide/mergedProjects/analyzer_ec/mergedProjects/reference_olh/mergedProjects/instructions/instruct32_hh/vc35.htm
[CSEL]: https://developer.arm.com/documentation/dui0802/b/CSEL
[predication]: https://en.wikipedia.org/wiki/Predication_(computer_architecture)
[x86-cmov-conversion]: https://dsprenkels.com/cmov-conversion.html
