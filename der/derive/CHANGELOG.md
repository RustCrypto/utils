# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.0 (2021-06-07)
### Changed
- Generated code updates which ensure compatibility with upstream `der` crate
  changes ([#464], [#465], [#481])

[#464]: https://github.com/RustCrypto/utils/pull/464
[#465]: https://github.com/RustCrypto/utils/pull/465
[#481]: https://github.com/RustCrypto/utils/pull/481

## 0.3.0 (2021-03-21)
### Added
- `choice::Alternative` and duplicate tracking ([#300])
- Auto-derive `From` impls for variants when deriving `Choice` ([#345])

[#300]: https://github.com/RustCrypto/utils/pull/300
[#345]: https://github.com/RustCrypto/utils/pull/345

## 0.2.2 (2021-02-22)
### Added
- Custom derive support for the `Choice` trait ([#296])

[#296]: https://github.com/RustCrypto/utils/pull/296

## 0.2.1 (2021-02-15)
### Added
- Custom derive support for enums ([#254])

[#254]: https://github.com/RustCrypto/utils/pull/254

## 0.2.0 (2021-02-02)
### Added
- Support for `PrintableString` and `Utf8String` ([#245])

[#245]: https://github.com/RustCrypto/utils/pull/245

## 0.1.0 (2020-12-21)
- Initial release
