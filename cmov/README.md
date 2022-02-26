# [RustCrypto]: Conditional Move Intrinsics

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][rustc-image]
[![Build Status][build-image]][build-link]

Conditional move CPU intrinsics which are guaranteed to execute in
constant-time and not be rewritten as branches by the compiler.

Provides wrappers for the [CMOV family] of instructions on x86/x86_64 CPUs.

[Documentation][docs-link]

## About

Conditional move intrinsics are a form of [predication] which allows selection
of one or more values without using branch instructions, thus making the
selection constant-time with respect to the values, and not subject to CPU
execution features which might introduce timing or other microarchitectural
sidechannels introduced by branch prediction or other speculative execution
features.

Intel has confirmed that all extant CPUs implement the CMOV family of
instructions in constant-time, and that this property will hold for future
Intel CPUs as well.

This crate provides wrappers for the CMOV instructions implemented in terms
of inline assembly as stabilized in Rust 1.59. This means the implementation
is a black box that will not be rewritten by e.g. LLVM's architecture-specific
lowerings, such as the [x86-cmov-conversion] pass.

## Supported target architectures

This crate will only compile on the following target architectures:

- [x] `x86`
- [x] `x86_64`
- [ ] `aarch64`
- [ ] Others?

Use the following syntax in `Cargo.toml` to conditionally include it:

```toml
[target.'cfg(any(target_arch = "x86", target_arch = "x86"))'.dependencies]
cmov = "0"
```

Please open an issue inquiring about support for other architectures.

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
[rustc-image]: https://img.shields.io/badge/rustc-1.51+-blue.svg
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/cmov.yml/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/cmov.yml

[//]: # (general links)

[RustCrypto]: https://github.com/RustCrypto
[CMOV family]: https://www.jaist.ac.jp/iscenter-new/mpc/altix/altixdata/opt/intel/vtune/doc/users_guide/mergedProjects/analyzer_ec/mergedProjects/reference_olh/mergedProjects/instructions/instruct32_hh/vc35.htm
[predication]: https://en.wikipedia.org/wiki/Predication_(computer_architecture)
[x86-cmov-conversion]: https://dsprenkels.com/cmov-conversion.html
