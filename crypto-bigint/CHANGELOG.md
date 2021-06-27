# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.2 (2021-06-26)
### Added
- `Limb::is_odd` and `UInt::is_odd` ([#505])
- `UInt::new` ([#506])
- `rand` feature ([#508])

### Changed
- Deprecate `LIMB_BYTES` constant ([#504])
- Make `Limb`'s `Inner` value public ([#507])

[#504]: https://github.com/RustCrypto/utils/pull/504
[#505]: https://github.com/RustCrypto/utils/pull/505
[#506]: https://github.com/RustCrypto/utils/pull/506
[#507]: https://github.com/RustCrypto/utils/pull/507
[#508]: https://github.com/RustCrypto/utils/pull/508

## 0.2.1 (2021-06-21)
### Added
- `Limb` newtype ([#499])
- Target-specific rustdocs ([#500])

[#499]: https://github.com/RustCrypto/utils/pull/499
[#500]: https://github.com/RustCrypto/utils/pull/500

## 0.2.0 (2021-06-07) [YANKED]
### Added
- `ConstantTimeGreater`/`ConstantTimeLess` impls for UInt ([#459])
- `From` conversions between `UInt` and limb arrays ([#460])
- `zeroize` feature ([#461])
- Additional `ArrayEncoding::ByteSize` bounds ([#462])
- `UInt::into_limbs` ([#484])
- `Encoding` trait ([#488])

### Removed
- `NumBits`/`NumBytes` traits; use `Encoding` instead ([#488])

[#459]: https://github.com/RustCrypto/utils/pull/459
[#460]: https://github.com/RustCrypto/utils/pull/460
[#461]: https://github.com/RustCrypto/utils/pull/461
[#462]: https://github.com/RustCrypto/utils/pull/462
[#484]: https://github.com/RustCrypto/utils/pull/484
[#488]: https://github.com/RustCrypto/utils/pull/488

## 0.1.0 (2021-05-30)
- Initial release
