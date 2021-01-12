# RustCrypto: B64 encoding

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]
[![Build Status][build-image]][build-link]

Pure Rust implementation of [B64], a subset of the standard Base64 encoding
([RFC 4648]) used by the [PHC string format].

Implemented without data-dependent branches or look up tables, thereby
providing "best effort" constant-time operation.

Supports `no_std` environments and avoids heap allocations in the core API
(but also provides optional `alloc` support for convenience).

[Documentation][docs-link]

## About B64

The following description of [B64] is quoted from the [PHC string format] spec:

> The B64 encoding is the standard Base64 encoding (RFC 4648, section 4)
> except that the padding `=` signs are omitted, and extra characters
> (whitespace) are not allowed:
>
> - Input is split into successive groups of bytes. Each group, except
>   possibly the last one, contains exactly three bytes.
>
> - For a group of bytes b0, b1 and b2, compute the following value:
>
>         x = (b0 << 16) + (b1 << 8) + b2
>
>  Then split `x` into four 6-bit values `y0`, `y1`, `y2` and `y3`
>  such that:
>
>         x = (y0 << 18) + (y1 << 12) + (y2 << 6) + y3
>
> - Each 6-bit value is encoded into a character in the `[A-Za-z0-9+/]`
>   alphabet, in that order:
>    - `A`..`Z` = 0 to 25
>    - `a`..`z` = 26 to 51
>    - `0`..`9` = 52 to 61
>    - `+` = 62
>    - `/` = 63
>
> - If the last group does not contain exactly three bytes, then:
>
>    1. The group is completed with one or two bytes of value 0x00,
>       then processed as above.
>    2. The resulting sequence of characters is truncated to its
>       first two characters (if the group initially contained a single
>       byte) or to its first three characters (if the group initially
>       contained two bytes).
>
> A B64-encoded value thus yields a string whose length, taken modulo 4,
> can be equal to 0, 2 or 3, but not to 1. Take note that a sequence of
> characters of the right length may still be an invalid encoding if it
> defines some non-zero trailing bits in the last incomplete group;
> producers MUST set the trailing bits to 0, while consumers MAY ignore
> them, or MAY reject such invalid encodings.

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

[crate-image]: https://img.shields.io/crates/v/b64ct.svg
[crate-link]: https://crates.io/crates/b64ct
[docs-image]: https://docs.rs/b64ct/badge.svg
[docs-link]: https://docs.rs/b64ct/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.41+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/workflows/b64ct/badge.svg?branch=master&event=push
[build-link]: https://github.com/RustCrypto/utils/actions?query=workflow:b64ct

[//]: # (general links)

[B64]: https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md#b64
[RFC 4648]: https://tools.ietf.org/html/rfc4648
[PHC string format]: https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md
