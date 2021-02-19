# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.2 (2021-02-19)
### Fixed
- Bug in root arc calculation ([#284])

[#284]: https://github.com/RustCrypto/utils/pull/284

## 0.4.1 (2020-12-21)
### Fixed
- Bug in const initializer ([#172])

[#172]: https://github.com/RustCrypto/utils/pull/172

## 0.4.0 (2020-12-16)
### Added
- `Arcs` iterator ([#141], [#142])

### Changed
- Rename "nodes" to "arcs" ([#142])
- Layout optimization ([#143])
- Refactor and improve length limits ([#144])

[#144]: https://github.com/RustCrypto/utils/pull/144
[#143]: https://github.com/RustCrypto/utils/pull/143
[#142]: https://github.com/RustCrypto/utils/pull/142
[#141]: https://github.com/RustCrypto/utils/pull/141

## 0.3.5 (2020-12-12)
### Added
- `ObjectIdentifier::{write_ber, to_ber}` methods ([#118])

[#118]: https://github.com/RustCrypto/utils/pull/118

## 0.3.4 (2020-12-06)
### Changed
- Documentation improvements ([#112])

[#112]: https://github.com/RustCrypto/utils/pull/110

## 0.3.3 (2020-12-05)
### Changed
- Improve description in Cargo.toml/README.md (#110)

[#110]: https://github.com/RustCrypto/utils/pull/110

## 0.3.2 (2020-12-05)
### Changed
- Documentation improvements ([#107])

[#107]: https://github.com/RustCrypto/utils/pull/107

## 0.3.1 (2020-12-05)
### Added
- Impl `TryFrom<&[u32]>` for ObjectIdentifier ([#105])

[#105]: https://github.com/RustCrypto/utils/pull/105

## 0.3.0 (2020-12-05) [YANKED]
### Added
- Byte and string parsers ([#89])

[#89]: https://github.com/RustCrypto/utils/pull/89

## 0.2.0 (2020-09-05)
### Changed
- Validate OIDs are well-formed; MSRV 1.46+ ([#76])

[#76]: https://github.com/RustCrypto/utils/pull/76

## 0.1.0 (2020-08-04)
- Initial release
