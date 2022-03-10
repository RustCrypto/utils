# RustCrypto: Utilities

[![Project Chat][chat-image]][chat-link]
[![dependency status][deps-image]][deps-link]
![Apache2/MIT licensed][license-image]

This repository contains various utility crates used in the RustCrypto project.

## Crates

| Name | crates.io | Docs | MSRV | Description |
|------|:---------:|:----:|:----:|-------------|
| [`blobby`] | [![crates.io](https://img.shields.io/crates/v/blobby.svg)](https://crates.io/crates/blobby) | [![Documentation](https://docs.rs/blobby/badge.svg)](https://docs.rs/blobby) | ![MSRV 1.39][msrv-1.39] | Decoder of the simple de-duplicated binary blob storage format |
| [`block-buffer`] | [![crates.io](https://img.shields.io/crates/v/block-buffer.svg)](https://crates.io/crates/block-buffer) | [![Documentation](https://docs.rs/block-buffer/badge.svg)](https://docs.rs/block-buffer) | ![MSRV 1.41][msrv-1.41] | Fixed size buffer for block processing of data |
| [`block‑padding`] | [![crates.io](https://img.shields.io/crates/v/block-padding.svg)](https://crates.io/crates/block-padding) | [![Documentation](https://docs.rs/block-padding/badge.svg)](https://docs.rs/block-padding) | ![MSRV 1.56][msrv-1.56] | Padding and unpadding of messages divided into blocks |
| [`cmov`] | [![crates.io](https://img.shields.io/crates/v/cmov.svg)](https://crates.io/crates/cmov) | [![Documentation](https://docs.rs/cmov/badge.svg)](https://docs.rs/cmov) | ![MSRV 1.59][msrv-1.59] | Conditional move intrinsics |
| [`collectable`] | [![crates.io](https://img.shields.io/crates/v/collectable.svg)](https://crates.io/crates/collectable) | [![Documentation](https://docs.rs/collectable/badge.svg)](https://docs.rs/collectable) | ![MSRV 1.41][msrv-1.41] | Fallible, `no_std`-friendly collection traits |
| [`cpufeatures`] | [![crates.io](https://img.shields.io/crates/v/cpufeatures.svg)](https://crates.io/crates/cpufeatures) | [![Documentation](https://docs.rs/cpufeatures/badge.svg)](https://docs.rs/cpufeatures) | ![MSRV 1.40][msrv-1.40] | Lightweight and efficient alternative to the `is_x86_feature_detected!` macro |
| [`dbl`] | [![crates.io](https://img.shields.io/crates/v/dbl.svg)](https://crates.io/crates/dbl) | [![Documentation](https://docs.rs/dbl/badge.svg)](https://docs.rs/dbl) | ![MSRV 1.41][msrv-1.41] | Double operation in Galois Field (GF) |
| [`hex-literal`] | [![crates.io](https://img.shields.io/crates/v/hex-literal.svg)](https://crates.io/crates/hex-literal) | [![Documentation](https://docs.rs/hex-literal/badge.svg)](https://docs.rs/hex-literal) | ![MSRV 1.45][msrv-1.45] | Procedural macro for converting hexadecimal string to byte array at compile time |
| [`inout`] | [![crates.io](https://img.shields.io/crates/v/inout.svg)](https://crates.io/crates/inout) | [![Documentation](https://docs.rs/inout/badge.svg)](https://docs.rs/inout) | ![MSRV 1.56][msrv-1.56] | Custom reference types for code generic over in-place and buffer-to-buffer modes of operation. |
| [`opaque-debug`] | [![crates.io](https://img.shields.io/crates/v/opaque-debug.svg)](https://crates.io/crates/opaque-debug) | [![Documentation](https://docs.rs/opaque-debug/badge.svg)](https://docs.rs/opaque-debug) | ![MSRV 1.41][msrv-1.41] | Macro for opaque `Debug` trait implementation |
| [`wycheproof2blb`] |  |  | | Utility for converting [Wycheproof] test vectors to the blobby format |
| [`zeroize`] | [![crates.io](https://img.shields.io/crates/v/zeroize.svg)](https://crates.io/crates/zeroize) | [![Documentation](https://docs.rs/zeroize/badge.svg)](https://docs.rs/zeroize) | ![MSRV 1.51][msrv-1.51] | Securely zero memory while avoiding compiler optimizations |

## License

All crates licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[deps-image]: https://deps.rs/repo/github/RustCrypto/utils/status.svg
[deps-link]: https://deps.rs/repo/github/RustCrypto/utils

[msrv-1.39]: https://img.shields.io/badge/rustc-1.39.0+-blue.svg
[msrv-1.40]: https://img.shields.io/badge/rustc-1.40.0+-blue.svg
[msrv-1.41]: https://img.shields.io/badge/rustc-1.41.0+-blue.svg
[msrv-1.45]: https://img.shields.io/badge/rustc-1.45.0+-blue.svg
[msrv-1.51]: https://img.shields.io/badge/rustc-1.51.0+-blue.svg
[msrv-1.56]: https://img.shields.io/badge/rustc-1.56.0+-blue.svg
[msrv-1.59]: https://img.shields.io/badge/rustc-1.59.0+-blue.svg

[//]: # (crates)

[`blobby`]: ./blobby
[`block-buffer`]: ./block-buffer
[`block‑padding`]: ./block-padding
[`cmov`]: ./cmov
[`collectable`]: ./collectable
[`cpufeatures`]: ./cpufeatures
[`dbl`]: ./dbl
[`hex-literal`]: ./hex-literal
[`inout`]: ./inout
[`opaque-debug`]: ./opaque-debug
[`wycheproof2blb`]: ./wycheproof2blb
[`zeroize`]: ./zeroize

[//]: # (misc)

[Wycheproof]: https://github.com/google/wycheproof
