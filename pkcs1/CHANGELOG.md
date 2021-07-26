# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.3 (2021-07-26)
### Added
- Support for customizing PEM `LineEnding` ([#553])

### Changed
- Bump `pem-rfc7468` dependency to v0.2 ([#552])

[#552]: https://github.com/RustCrypto/utils/pull/552
[#553]: https://github.com/RustCrypto/utils/pull/553

## 0.2.2 (2021-07-25)
### Fixed
- `Version` encoder ([#547])

[#547]: https://github.com/RustCrypto/utils/pull/547

## 0.2.1 (2021-07-25)
### Added
- `Error::Crypto` variant ([#544])

[#544]: https://github.com/RustCrypto/utils/pull/544

## 0.2.0 (2021-07-25)
### Added
- `From*`/`To*` traits for `RsaPrivateKey`/`RsaPublicKey` ([#540])

### Changed
- Use `FromRsa*`/`ToRsa*` traits with `*Document` types ([#541])

[#540]: https://github.com/RustCrypto/utils/pull/540
[#541]: https://github.com/RustCrypto/utils/pull/541

## 0.1.1 (2021-07-24)
### Added
- Re-export `der` crate and `der::UIntBytes` ([#537])

### Changed
- Replace `Error::{Decode, Encode}` with `Error::Asn1` ([#538])

[#537]: https://github.com/RustCrypto/utils/pull/537
[#538]: https://github.com/RustCrypto/utils/pull/538

## 0.1.0 (2021-07-24) [YANKED]
- Initial release
