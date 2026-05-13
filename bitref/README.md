# [RustCrypto]: Bit-Level Reference Types

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache 2.0/MIT Licensed][license-image]
![MSRV][msrv-image]
[![Project Chat][chat-image]][chat-link]

Provides `&BitSlice`/`&mut BitSlice`, a fat pointer-sized reference type which can be initialized
from `&[u8]`/`&mut [u8]` and can be used to implement any reference-based patterns that possible
with byte slices, e.g. `Borrow`, `Deref`, `Index`/`IndexMut`, and `ToOwned`.

## About

The `BitSlice` type in this crate was inspired by a similar type in the [`bitvec`]. However, the
implementation approach used in this crate minimizes use of `unsafe` code to just fat pointer
encoding/decoding and simple `repr(transparent)` casts to construct reference newtypes.

Notably it performs no arithmetic on pointers whatsoever and works entirely within the domain
of slices, largely ensuring any bugs should result in panics rather than memory safety errors,
with a hypothetical caveat noted below.

(NOTE: in the future, we may experiment with more usage of `unsafe` if the performance gains can
justify its use, but will ensure the resulting code is easily reasoned about)

## Soundness

This crate relies on operations which are not yet fully specified in the Rust memory model, and
while sound in all existing supported versions of the Rust compiler, may result in undefined
behavior in future versions.

Thus this crate is considered EXPERIMENTAL, and while it's been written with the intent of future
use in cryptographic applications by minimizing use of `unsafe` and ensuring an otherwise simple and
minimal implementation, until this situation changes it should not be considered ready for
production use.

CI checks the crate is sound under Miri with `-Zmiri-tree-borrows -Zmiri-strict-provenance` which
checks the code under the Tree Borrows model, however it is known to fail under Stacked Borrows.
More information can be found in the SAFETY comments in the source code.

Ensuring this crate will be fully sound in future versions of Rust will require upstream resolution
regarding the operational semantics and this discrepancy between SB/TB. For more information, see:

<https://github.com/rust-lang/unsafe-code-guidelines/issues/134>

## Minimum Supported Rust Version (MSRV) Policy

MSRV increases are not considered breaking changes and can happen in patch releases.

The crate MSRV accounts for all supported targets and crate feature combinations.

## License

Licensed under either of:

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/bitref.svg
[crate-link]: https://crates.io/crates/bitref
[docs-image]: https://docs.rs/bitref/badge.svg
[docs-link]: https://docs.rs/bitref/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[msrv-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/bitref.yml/badge.svg
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/bitref.yml
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils

[//]: # (links)

[RustCrypto]: https://github.com/RustCrypto
[`bitvec`]: https://docs.rs/bitvec
