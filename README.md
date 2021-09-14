# RustCrypto: Utilities [![Project Chat][chat-image]][chat-link] [![dependency status][deps-image]][deps-link]

This repository contains various utility crates used in the RustCrypto project.

## Crates

| Name | crates.io | Docs | Description |
|------|-----------|------|--------------|
| `blobby` | [![crates.io](https://img.shields.io/crates/v/blobby.svg)](https://crates.io/crates/blobby) | [![Documentation](https://docs.rs/blobby/badge.svg)](https://docs.rs/blobby) | Decoder of the simple de-duplicated binary blob storage format |
| `block-buffer` | [![crates.io](https://img.shields.io/crates/v/block-buffer.svg)](https://crates.io/crates/block-buffer) | [![Documentation](https://docs.rs/block-buffer/badge.svg)](https://docs.rs/block-buffer) | Fixed size buffer for block processing of data |
| `block‑padding` | [![crates.io](https://img.shields.io/crates/v/block-padding.svg)](https://crates.io/crates/block-padding) | [![Documentation](https://docs.rs/block-padding/badge.svg)](https://docs.rs/block-padding) | Padding and unpadding of messages divided into blocks |
| `collectable` | [![crates.io](https://img.shields.io/crates/v/collectable.svg)](https://crates.io/crates/collectable) | [![Documentation](https://docs.rs/collectable/badge.svg)](https://docs.rs/collectable) | Fallible, `no_std`-friendly collection traits |
| `cpufeatures` | [![crates.io](https://img.shields.io/crates/v/cpufeatures.svg)](https://crates.io/crates/cpufeatures) | [![Documentation](https://docs.rs/cpufeatures/badge.svg)](https://docs.rs/cpufeatures) | Lightweight and efficient alternative to the `is_x86_feature_detected!` macro |
| `crypto‑bigint` | [![crates.io](https://img.shields.io/crates/v/crypto-bigint.svg)](https://crates.io/crates/crypto-bigint) | [![Documentation](https://docs.rs/crypto-bigint/badge.svg)](https://docs.rs/crypto-bigint) | Big integer library for cryptographic applciations |
| `dbl` | [![crates.io](https://img.shields.io/crates/v/dbl.svg)](https://crates.io/crates/dbl) | [![Documentation](https://docs.rs/dbl/badge.svg)](https://docs.rs/dbl) | Double operation in Galois Field (GF) |
| `hex-literal` | [![crates.io](https://img.shields.io/crates/v/hex-literal.svg)](https://crates.io/crates/hex-literal) | [![Documentation](https://docs.rs/hex-literal/badge.svg)](https://docs.rs/hex-literal) | Procedural macro for converting hexadecimal string to byte array at compile time |
| `opaque-debug` | [![crates.io](https://img.shields.io/crates/v/opaque-debug.svg)](https://crates.io/crates/opaque-debug) | [![Documentation](https://docs.rs/opaque-debug/badge.svg)](https://docs.rs/opaque-debug) | Macro for opaque `Debug` trait implementation |

## License

All crates licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[deps-image]: https://deps.rs/repo/github/RustCrypto/utils/status.svg
[deps-link]: https://deps.rs/repo/github/RustCrypto/utils
