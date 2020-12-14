# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.1 (2020-12-14)
### Added
- rustdoc improvements ([#130])

[#130]: https://github.com/RustCrypto/utils/pull/130

## 0.2.0 (2020-12-14)
### Added
- File writing methods for public/private keys ([#126])
- Methods for loading `*Document` types from files ([#125])
- DER encoding support ([#120], [#121])
- PEM encoding support ([#122], [#124])
- `ToPrivateKey`/`ToPublicKey` traits ([#123])

### Changed
- `Error` enum ([#128])
- Rename `load_*_file` methods to `read_*_file` ([#127])

[#128]: https://github.com/RustCrypto/utils/pull/128
[#127]: https://github.com/RustCrypto/utils/pull/127
[#126]: https://github.com/RustCrypto/utils/pull/126
[#125]: https://github.com/RustCrypto/utils/pull/125
[#124]: https://github.com/RustCrypto/utils/pull/124
[#123]: https://github.com/RustCrypto/utils/pull/123
[#122]: https://github.com/RustCrypto/utils/pull/122
[#121]: https://github.com/RustCrypto/utils/pull/121
[#120]: https://github.com/RustCrypto/utils/pull/120

## 0.1.1 (2020-12-06)
### Added
- Helper methods to load keys from the local filesystem ([#115])

[#115]: https://github.com/RustCrypto/utils/pull/115

## 0.1.0 (2020-12-05)
- Initial release
