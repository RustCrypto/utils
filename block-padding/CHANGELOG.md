# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.1 (2022-02-10)
### Fixed
- Fix doc build on docs.rs by optionally enabling the `doc_cfg` feature ([#733])

[#733]: https://github.com/RustCrypto/utils/pull/733

## 0.3.0 (2022-02-10)
### Added
- `Iso10126` padding algorithm ([#643])
- `PadType` enum, `Padding::TYPE` associated constant, and `Padding::unpad_blocks` method ([#675])

### Changed
- The `Padding` trait methods now work with blocks instead of byte slices. ([#113])
- Bump MSRV to 1.56 and edition to 2021  ([#675])

[#113]: https://github.com/RustCrypto/utils/pull/113
[#643]: https://github.com/RustCrypto/utils/pull/643
[#675]: https://github.com/RustCrypto/utils/pull/675

## 0.2.1 (2020-08-14)
### Added
- `Copy`, `Clone`, and `Debug` trait implementations for padding types. ([#78])

[#78]: https://github.com/RustCrypto/utils/pull/78

## 0.2.0 (2020-07-10)