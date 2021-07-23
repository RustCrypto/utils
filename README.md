# RustCrypto: Utilities [![Project Chat][chat-image]][chat-link] [![dependency status][deps-image]][deps-link]

This repository contains various utility crates used in the RustCrypto project.

## Crates

| Name | crates.io | Docs | Description |
|------|-----------|------|--------------|
| `base64ct` | [![crates.io](https://img.shields.io/crates/v/base64ct.svg)](https://crates.io/crates/base64ct) | [![Documentation](https://docs.rs/base64ct/badge.svg)](https://docs.rs/base64ct) | Constant-time encoder and decoder of several Base64 variants |
| `blobby` | [![crates.io](https://img.shields.io/crates/v/blobby.svg)](https://crates.io/crates/blobby) | [![Documentation](https://docs.rs/blobby/badge.svg)](https://docs.rs/blobby) | Decoder of the simple de-duplicated binary blob storage format |
| `block-buffer` | [![crates.io](https://img.shields.io/crates/v/block-buffer.svg)](https://crates.io/crates/block-buffer) | [![Documentation](https://docs.rs/block-buffer/badge.svg)](https://docs.rs/block-buffer) | Fixed size buffer for block processing of data |
| `block‑padding` | [![crates.io](https://img.shields.io/crates/v/block-padding.svg)](https://crates.io/crates/block-padding) | [![Documentation](https://docs.rs/block-padding/badge.svg)](https://docs.rs/block-padding) | Padding and unpadding of messages divided into blocks |
| `collectable` | [![crates.io](https://img.shields.io/crates/v/collectable.svg)](https://crates.io/crates/collectable) | [![Documentation](https://docs.rs/collectable/badge.svg)](https://docs.rs/collectable) | Fallible, `no_std`-friendly collection traits |
| `const-oid` | [![crates.io](https://img.shields.io/crates/v/const-oid.svg)](https://crates.io/crates/const-oid) | [![Documentation](https://docs.rs/const-oid/badge.svg)](https://docs.rs/const-oid) | Const-friendly implementation of the ISO/IEC Object Identifier (OID) standard as defined in [ITU X.660] |
| `cpufeatures` | [![crates.io](https://img.shields.io/crates/v/cpufeatures.svg)](https://crates.io/crates/cpufeatures) | [![Documentation](https://docs.rs/cpufeatures/badge.svg)](https://docs.rs/cpufeatures) | Lightweight and efficient alternative to the `is_x86_feature_detected!` macro |
| `crypto‑bigint` | [![crates.io](https://img.shields.io/crates/v/crypto-bigint.svg)](https://crates.io/crates/crypto-bigint) | [![Documentation](https://docs.rs/crypto-bigint/badge.svg)](https://docs.rs/crypto-bigint) | Big integer library for cryptographic applciations |
| `dbl` | [![crates.io](https://img.shields.io/crates/v/dbl.svg)](https://crates.io/crates/dbl) | [![Documentation](https://docs.rs/dbl/badge.svg)](https://docs.rs/dbl) | Double operation in Galois Field (GF) |
| `der` | [![crates.io](https://img.shields.io/crates/v/der.svg)](https://crates.io/crates/der) | [![Documentation](https://docs.rs/der/badge.svg)](https://docs.rs/der) | Decoder and encoder of the Distinguished Encoding Rules (DER) for Abstract Syntax Notation One (ASN.1) as described in [ITU X.690] |
| `hex-literal` | [![crates.io](https://img.shields.io/crates/v/hex-literal.svg)](https://crates.io/crates/hex-literal) | [![Documentation](https://docs.rs/hex-literal/badge.svg)](https://docs.rs/hex-literal) | Procedural macro for converting hexadecimal string to byte array at compile time |
| `opaque-debug` | [![crates.io](https://img.shields.io/crates/v/opaque-debug.svg)](https://crates.io/crates/opaque-debug) | [![Documentation](https://docs.rs/opaque-debug/badge.svg)](https://docs.rs/opaque-debug) | Macro for opaque `Debug` trait implementation |
| `pem-rfc7468` | [![crates.io](https://img.shields.io/crates/v/pem-rfc7468.svg)](https://crates.io/crates/pem-rfc7468) | [![Documentation](https://docs.rs/pem-rfc7468/badge.svg)](https://docs.rs/pem-rfc7468) | Strict PEM encoding for PKIX/PKCS/CMS objects |
| `pkcs1` | [![crates.io](https://img.shields.io/crates/v/pkcs1.svg)](https://crates.io/crates/pkcs1) | [![Documentation](https://docs.rs/pkcs1/badge.svg)](https://docs.rs/pkcs1) | Implementation of PKCS#1: RSA Cryptography Specifications Version 2.2 ([RFC 8017]) |
| `pkcs5` | [![crates.io](https://img.shields.io/crates/v/pkcs5.svg)](https://crates.io/crates/pkcs5) | [![Documentation](https://docs.rs/pkcs5/badge.svg)](https://docs.rs/pkcs5) | Implementation of PKCS#5: Password-Based Cryptography Specification Version 2.1 ([RFC 8018]) |
| `pkcs8` | [![crates.io](https://img.shields.io/crates/v/pkcs8.svg)](https://crates.io/crates/pkcs8) | [![Documentation](https://docs.rs/pkcs8/badge.svg)](https://docs.rs/pkcs8) | Implementation of PKCS#8(v2): Private-Key Information Syntax Specification ([RFC 5208]) and asymmetric key packages ([RFC 5958]) |
| `spki` | [![crates.io](https://img.shields.io/crates/v/spki.svg)](https://crates.io/crates/spki) | [![Documentation](https://docs.rs/spki/badge.svg)](https://docs.rs/spki) | X.509 Subject Public Key Info ([RFC 5280 Section 4.1]) describing public keys as well as their associated AlgorithmIdentifiers (i.e. OIDs) |
| `x509` | [![crates.io](https://img.shields.io/crates/v/x509.svg)](https://crates.io/crates/x509) | [![Documentation](https://docs.rs/x509/badge.svg)](https://docs.rs/x509) | Implementation of the X.509 Public Key Infrastructure Certificate format as described in [RFC 5280] |

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

[ITU X.660]: https://www.itu.int/rec/T-REC-X.660
[ITU X.690]: https://www.itu.int/rec/T-REC-X.690
[RFC 8017]: https://datatracker.ietf.org/doc/html/rfc8017
[RFC 8018]: https://datatracker.ietf.org/doc/html/rfc8018
[RFC 5208]: https://datatracker.ietf.org/doc/html/rfc5208
[RFC 5958]: https://datatracker.ietf.org/doc/html/rfc5958
[RFC 5280 Section 4.1]: https://datatracker.ietf.org/doc/html/rfc5280#section-4.1
[RFC 5280]: https://datatracker.ietf.org/doc/html/rfc5280
