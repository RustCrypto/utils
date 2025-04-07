# RustCrypto: Utilities

[![Project Chat][chat-image]][chat-link]
[![dependency status][deps-image]][deps-link]
![Apache2/MIT licensed][license-image]

This repository contains various utility crates used in the RustCrypto project.

## Crates

| Name | crates.io | Docs | Description |
|------|:---------:|:----:|-------------|
| [`blobby`] | [![crates.io](https://img.shields.io/crates/v/blobby.svg)](https://crates.io/crates/blobby) | [![Documentation](https://docs.rs/blobby/badge.svg)](https://docs.rs/blobby) | Decoder of the simple de-duplicated binary blob storage format |
| [`block-buffer`] | [![crates.io](https://img.shields.io/crates/v/block-buffer.svg)](https://crates.io/crates/block-buffer) | [![Documentation](https://docs.rs/block-buffer/badge.svg)](https://docs.rs/block-buffer) | Fixed size buffer for block processing of data |
| [`block‑padding`] | [![crates.io](https://img.shields.io/crates/v/block-padding.svg)](https://crates.io/crates/block-padding) | [![Documentation](https://docs.rs/block-padding/badge.svg)](https://docs.rs/block-padding) | Padding and unpadding of messages divided into blocks |
| [`cmov`] | [![crates.io](https://img.shields.io/crates/v/cmov.svg)](https://crates.io/crates/cmov) | [![Documentation](https://docs.rs/cmov/badge.svg)](https://docs.rs/cmov) | Conditional move intrinsics |
| [`collectable`] | [![crates.io](https://img.shields.io/crates/v/collectable.svg)](https://crates.io/crates/collectable) | [![Documentation](https://docs.rs/collectable/badge.svg)](https://docs.rs/collectable) | Fallible, `no_std`-friendly collection traits |
| [`cpufeatures`] | [![crates.io](https://img.shields.io/crates/v/cpufeatures.svg)](https://crates.io/crates/cpufeatures) | [![Documentation](https://docs.rs/cpufeatures/badge.svg)](https://docs.rs/cpufeatures) | Lightweight and efficient alternative to the `is_x86_feature_detected!` macro |
| [`dbl`] | [![crates.io](https://img.shields.io/crates/v/dbl.svg)](https://crates.io/crates/dbl) | [![Documentation](https://docs.rs/dbl/badge.svg)](https://docs.rs/dbl) | Double operation in Galois Field (GF) |
| [`digest-io`] | [![crates.io](https://img.shields.io/crates/v/digest-io.svg)](https://crates.io/crates/digest-io) | [![Documentation](https://docs.rs/digest-io/badge.svg)](https://docs.rs/digest-io) | `std::io`-compatibility wrappers for traits defined in the `digest` crate |
| [`hex-literal`] | [![crates.io](https://img.shields.io/crates/v/hex-literal.svg)](https://crates.io/crates/hex-literal) | [![Documentation](https://docs.rs/hex-literal/badge.svg)](https://docs.rs/hex-literal) | A macro for converting hexadecimal strings to a byte array at compile time |
| [`inout`] | [![crates.io](https://img.shields.io/crates/v/inout.svg)](https://crates.io/crates/inout) | [![Documentation](https://docs.rs/inout/badge.svg)](https://docs.rs/inout) | Custom reference types for code generic over in-place and buffer-to-buffer modes of operation. |
| [`opaque-debug`] | [![crates.io](https://img.shields.io/crates/v/opaque-debug.svg)](https://crates.io/crates/opaque-debug) | [![Documentation](https://docs.rs/opaque-debug/badge.svg)](https://docs.rs/opaque-debug) | Macro for opaque `Debug` trait implementation |
| [`wycheproof2blb`] |  |  | | Utility for converting [Wycheproof] test vectors to the blobby format |
| [`zeroize`] | [![crates.io](https://img.shields.io/crates/v/zeroize.svg)](https://crates.io/crates/zeroize) | [![Documentation](https://docs.rs/zeroize/badge.svg)](https://docs.rs/zeroize) | Securely zero memory while avoiding compiler optimizations |

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

[//]: # (crates)

[`blobby`]: ./blobby
[`block-buffer`]: ./block-buffer
[`block‑padding`]: ./block-padding
[`cmov`]: ./cmov
[`collectable`]: ./collectable
[`cpufeatures`]: ./cpufeatures
[`dbl`]: ./dbl
[`digest-io`]: ./digest-io
[`hex-literal`]: ./hex-literal
[`inout`]: ./inout
[`opaque-debug`]: ./opaque-debug
[`wycheproof2blb`]: ./wycheproof2blb
[`zeroize`]: ./zeroize

[//]: # (misc)

[Wycheproof]: https://github.com/google/wycheproof
