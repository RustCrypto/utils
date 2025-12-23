# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.1 (2025-12-19)
### Changed
- Use `black_box` in portable impl ([#1255])

[#1255]: https://github.com/RustCrypto/utils/pull/1255

## 0.4.0 (2025-09-10)
### Changed
- Edition changed to 2024 and MSRV bumped to 1.85 ([#1149])

[#1149]: https://github.com/RustCrypto/utils/pull/1149

## 0.3.1 (2023-10-14)
### Added
- `CmovEq` impl for slices ([#954])

### Changed
- Use `#[inline]` instead of `#[inline(always)]` ([#924])
- `CmovEq` now invokes XOR within the ASM block ([#925])

[#924]: https://github.com/RustCrypto/utils/pull/924
[#925]: https://github.com/RustCrypto/utils/pull/925
[#954]: https://github.com/RustCrypto/utils/pull/954

## 0.3.0 (2023-04-02)
### Added
- `miri` support by forcing the `portable` backend ([#864])
- Constant-time equality comparisons ([#873])

### Changed
- Make `Cmov::cmovz` a provided method ([#871])

### Fixed
- Builds on `x86` (32-bit) targets ([#863])

[#863]: https://github.com/RustCrypto/utils/pull/863
[#864]: https://github.com/RustCrypto/utils/pull/864
[#871]: https://github.com/RustCrypto/utils/pull/871
[#873]: https://github.com/RustCrypto/utils/pull/873

## 0.2.0 (2023-02-26)
### Added
- `Condition` alias for `u8` ([#830])

### Changed
- Redesigned trait-based API ([#830])
  - Built around a `Cmov` trait
  - Trait is impl'd for `u8`, `u16`, `u32`, `u64`, `u128`
  - Accepts a `Condition` (i.e. `u8`) as a predicate
- MSRV 1.60 ([#839])

[#830]: https://github.com/RustCrypto/utils/pull/830
[#839]: https://github.com/RustCrypto/utils/pull/839

## 0.1.1 (2022-03-02)
### Added
- `cmovz`/`cmovnz`-alike support for AArch64 targets ([#744])

[#744]: https://github.com/RustCrypto/utils/pull/744

## 0.1.0 (2022-02-27)
- Initial release
