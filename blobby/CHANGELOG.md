# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.4.0 (2025-10-08)
### Changed
- Edition changed to 2024 and MSRV bumped to 1.85 ([#1149])
- Replaced iterators with `const fn` parsing ([#1187])
- Format of the file. File header now contains total number of stored blobs. ([#1207])

[#1149]: https://github.com/RustCrypto/utils/pull/1149
[#1187]: https://github.com/RustCrypto/utils/pull/1187
[#1207]: https://github.com/RustCrypto/utils/pull/1207

## 0.3.1 (2021-12-07)
### Added
- `encode_blobs` function ([#280])

[#280]: https://github.com/RustCrypto/utils/pull/280

## 0.3.0 (2020-07-01)
### Changed
- New storage format with de-duplication capability ([#64])

[#64]: https://github.com/RustCrypto/utils/pull/64

## 0.2.0 (2020-06-13)
### Added
- `Blob5Iterator`

### Changed
- Bumped MSRV to 1.34.
- Removed `byteorder` from non-dev dependencies.

## 0.1.2 (2019-01-28)

## 0.1.1 (2018-09-26)

## 0.1.0 (2018-09-26)
- Initial release
