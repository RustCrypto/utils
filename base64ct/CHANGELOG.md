# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 1.0.1 (2021-08-14)
### Fixed
- Make `Encoding::decode` reject invalid padding ([#577])

[#577]: https://github.com/RustCrypto/utils/pull/577

## 1.0.0 (2021-03-17)
### Changed
- Bump MSRV to 1.47+ ([#334])

### Fixed
- MSRV-dependent TODOs in implementation ([#334])

[#334]: https://github.com/RustCrypto/utils/pull/334

## 0.2.1 (2021-03-07)
### Fixed
- MSRV docs ([#328])

[#328]: https://github.com/RustCrypto/utils/pull/328

## 0.2.0 (2021-02-01)
### Changed
- Refactor with `Encoding` trait ([#238])
- Internal refactoring ([#241], [#242])

[#238]: https://github.com/RustCrypto/utils/pull/238
[#241]: https://github.com/RustCrypto/utils/pull/241
[#242]: https://github.com/RustCrypto/utils/pull/242

## 0.1.2 (2021-01-31)
### Added
- bcrypt encoding ([#237])
- `crypt(3)` encoding ([#239])

### Changed
- Internal refactoring ([#235], [#236])

[#235]: https://github.com/RustCrypto/utils/pull/235
[#236]: https://github.com/RustCrypto/utils/pull/236
[#237]: https://github.com/RustCrypto/utils/pull/237
[#239]: https://github.com/RustCrypto/utils/pull/239

## 0.1.1 (2021-01-27)
- Minor code improvements ([#234])

[#234]: https://github.com/RustCrypto/utils/pull/234

## 0.1.0 (2021-01-26)
- Initial release
