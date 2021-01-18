# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.10.0 (2020-12-08)
### Changed
- Rename `input_block(s)` methods to `digest_block(s)`. ([#113])
- Upgrade the `block-padding` dependency to v0.3. ([#113])

### Added
- The `xor_data` method. ([#113])

### Removed
- The `input_lazy` method. ([#113])

[#113]: https://github.com/RustCrypto/utils/pull/113
