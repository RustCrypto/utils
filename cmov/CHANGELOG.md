# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
[#871]: https://github.com/RustCrypto/utils/pull/881
[#873]: https://github.com/RustCrypto/utils/pull/883

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
