# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## UNRELEASED
### Added
- `ReadBuffer` type ([#823])
- `serialize` and `deserialize` methods ([#823])

### Changed
- Supported block sizes are now bounded by the `crypto_common::BlockSizes` trait,
  which is implemented for types from `U1` to `U255` ([#823])
- Size of `EagerBuffer` is equal to buffer size, while previously it was equal
  to buffer size plus one byte ([#823])
- Edition changed to 2021 and MSRV bumped to 1.56 ([#823])

### Removed
- `EagerBuffer::set_data` method. Use the `ReadBuffer` type instead. ([#823])

[#823]: https://github.com/RustCrypto/utils/pull/823

## 0.10.3 (2022-09-04)
### Added
- `try_new` method ([#799])

[#799]: https://github.com/RustCrypto/utils/pull/799

## 0.10.2 (2021-02-08)
### Fixed
- Eliminate unreachable panic in `LazyBuffer::digest_blocks` ([#731])

[#731]: https://github.com/RustCrypto/utils/pull/731

## 0.10.1 (2021-02-05)
### Fixed
- Use `as_mut_ptr` to get a pointer for mutation in the `set_data` method ([#728])

[#728]: https://github.com/RustCrypto/utils/pull/728

## 0.10.0 (2020-12-07) [YANKED]
### Changed
- Significant reduction of number of unreachable panics. ([#671])
- Added buffer kind type parameter to `BlockBuffer`, respective marker types, and type aliases. ([#671])
- Various `BlockBuffer` method changes. ([#671])

### Removed
- `pad_with` method and dependency on `block-padding`. ([#671])

[#671]: https://github.com/RustCrypto/utils/pull/671

## 0.10.0 (2020-12-08)
### Changed
- Rename `input_block(s)` methods to `digest_block(s)`. ([#113])
- Upgrade the `block-padding` dependency to v0.3. ([#113])

### Added
- `par_xor_data`, `xor_data`, and `set_data` methods. ([#113])

### Removed
- The `input_lazy` method. ([#113])

[#113]: https://github.com/RustCrypto/utils/pull/113
