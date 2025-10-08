# [RustCrypto]: Blobby

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

An encoding and decoding library for the Blobby (`blb`) file format, which serves as a simple,
deduplicated storage format for a sequence of binary blobs.

## Examples
```rust
// We recommend to save blobby data into separate files and
// use the `include_bytes!` macro
static BLOBBY_DATA: &[u8; 27] = b"\x08\x02\x05hello\x06world!\x01\x02 \x00\x03\x06:::\x03\x01\x00";

static SLICE: &[&[u8]] = blobby::parse_into_slice!(BLOBBY_DATA);

assert_eq!(SLICE[0], b"hello".as_slice());
assert_eq!(SLICE[1], b" ".as_slice());
assert_eq!(SLICE[2], b"".as_slice());
assert_eq!(SLICE[3], b"world!".as_slice());
assert_eq!(SLICE[4], b":::".as_slice());
assert_eq!(SLICE[5], b"world!".as_slice());
assert_eq!(SLICE[6], b"hello".as_slice());
assert_eq!(SLICE[7], b"".as_slice());
assert_eq!(SLICE.len(), 8);

blobby::parse_into_structs!(
    BLOBBY_DATA;
    #[define_struct]
    static ITEMS: &[Item { a, b, c, d }];
);

assert_eq!(
    ITEMS[0],
    Item {
        a: b"hello",
        b: b" ",
        c: b"",
        d: b"world!",
    },
);
assert_eq!(
    ITEMS[1],
    Item {
        a: b":::",
        b: b"world!",
        c: b"hello",
        d: b"",
    },
);
assert_eq!(ITEMS.len(), 2);
```

## Encoding and decoding utilities

This crate provides encoding and decoding utilities for converting between
the blobby format and text file with hex-encoded strings. 

Let's say we have the following test vectors for a 64-bit hash function:
```text
COUNT = 0
INPUT = 0123456789ABCDEF0123456789ABCDEF
OUTPUT = 217777950848CECD

COUNT = 1
INPUT = 
OUTPUT = F7CD1446C9161C0A

COUNT = 2
INPUT = FFFEFD
OUTPUT = 80081C35AA43F640

```

To transform it into the Blobby format you first have to modify it
to the following format:

```text
0123456789ABCDEF0123456789ABCDEF
217777950848CECD

F7CD1446C9161C0A
FFFEFD
80081C35AA43F640

```
The first, third, and fifth lines are hex-encoded hash inputs, while the second,
fourth, and sixth lines are hex-encoded hash outputs for input on the previous line.
Note that the file should contain a trailing empty line (i.e. every data line should end
with `\n`).

This file can be converted to the Blobby format by running the following command:
```sh
cargo run --release --features alloc --bin encode -- /path/to/input.txt /path/to/output.blb
```

To inspect contents of an existing Blobby file you can use the following command:
```sh
cargo run --release --features alloc --bin decode -- /path/to/input.blb /path/to/output.txt
```
The output file will contain a sequence of hex-encoded byte strings stored
in the input file. 

## Storage format

Storage format represents a sequence of binary blobs. The format uses
git-flavored [variable-length quantity][VLQ] (VLQ) for encoding unsigned
numbers.

Blobby files start with two numbers: total number of blobs in the file `n` and
number of de-duplicated blobs `d`. The numbers are followed by `d` entries.
Each entry starts with an integer `m`, immediately followed by `m`
bytes representing de-duplicated binary blob.

Next, follows `n` entries representing sequence of stored blobs.
Each entry starts with an unsigned integer `l`. The least significant
bit of this integer is used as a flag. If the flag is equal to 0, then the
number is followed by `n >> 1` bytes, representing a stored binary blob.
Otherwise the entry references a de-duplicated entry number `n >> 1`
which should be smaller than `d`.

[VLQ]: https://en.wikipedia.org/wiki/Variable-length_quantity

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/blobby.svg
[crate-link]: https://crates.io/crates/blobby
[docs-image]: https://docs.rs/blobby/badge.svg
[docs-link]: https://docs.rs/blobby/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.85+-blue.svg
[chat-image]: https://img.shields.io/badge/zulip-join_chat-blue.svg
[chat-link]: https://rustcrypto.zulipchat.com/#narrow/stream/260052-utils
[build-image]: https://github.com/RustCrypto/utils/actions/workflows/blobby.yml/badge.svg?branch=master
[build-link]: https://github.com/RustCrypto/utils/actions/workflows/blobby.yml?query=branch:master

[//]: # (general links)

[RustCrypto]: https://github.com/rustcrypto
