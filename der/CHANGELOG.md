# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.6 (2021-02-19)
### Added
- Make the unit type an encoding of `NULL` ([#281])

[#281]: https://github.com/RustCrypto/utils/pull/281

## 0.2.5 (2021-02-18)
### Added
- `ErrorKind::UnknownOid` variant ([#273], [#275])

[#273]: https://github.com/RustCrypto/utils/pull/273
[#275]: https://github.com/RustCrypto/utils/pull/275

## 0.2.4 (2021-02-16)
### Added
- `Any::is_null` method ([#262])

### Changed
- Deprecate `Any::null` method ([#262])

[#262]: https://github.com/RustCrypto/utils/pull/262

## 0.2.3 (2021-02-15)
### Added
- Additional `rustdoc` documentation ([#252], [#256])

[#252]: https://github.com/RustCrypto/utils/pull/252
[#256]: https://github.com/RustCrypto/utils/pull/256

## 0.2.2 (2021-02-12)
### Added
- Support for `UTCTime` and `GeneralizedTime` ([#250])

[#250]: https://github.com/RustCrypto/utils/pull/250

## 0.2.1 (2021-02-02)
### Added
- Support for `PrintableString` and `Utf8String` ([#245])

[#245]: https://github.com/RustCrypto/utils/pull/245

## 0.2.0 (2021-01-22)
### Added
- `BigUInt` type ([#196])
- `i16` support ([#199])
- `u8` and `u16` support ([#210])
- Integer decoder helper methods ([#219])

### Fixed
- Handle leading byte of `BIT STRING`s ([#193])

[#193]: https://github.com/RustCrypto/utils/pull/193
[#196]: https://github.com/RustCrypto/utils/pull/196
[#199]: https://github.com/RustCrypto/utils/pull/199
[#210]: https://github.com/RustCrypto/utils/pull/210
[#219]: https://github.com/RustCrypto/utils/pull/219

## 0.1.0 (2020-12-21)
- Initial release
