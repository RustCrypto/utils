# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.3 (2025-12-26)
### Fixed
- Rustdoc syntax for variable-time-related warning text ([#1278])

[#1278]: https://github.com/RustCrypto/utils/pull/1278

## 0.1.2 (2025-12-26)
### Added
- Additional methods for `CtOption` ([#1274]):
  - `some`
  - `none`
  - `into_option_copied`
  - `filter_by`
  - `as_inner_unchecked`
  - `to_inner_unchecked`
- `Default` impl for `CtOption` ([#1274])
- `map!` and `unwrap_or!` macros ([#1274])
- `u128` methods for `Choice` ([#1277]):
  - `from_u128_le`
  - `from_u128_lsb`
  - `select_u128`

[#1274]: https://github.com/RustCrypto/utils/pull/1274
[#1277]: https://github.com/RustCrypto/utils/pull/1277

## 0.1.1 (2025-12-26)
### Added
- Additional `const fn` constructor and predication methods for `Choice` ([#1266], [#1272])

[#1266]: https://github.com/RustCrypto/utils/pull/1266
[#1272]: https://github.com/RustCrypto/utils/pull/1272

## 0.1.0 (2025-12-19)
- Initial release
