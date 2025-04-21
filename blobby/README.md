# [RustCrypto]: Blobby

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Project Chat][chat-image]][chat-link]

Iterators over a simple binary blob storage.

## Examples
```
let buf = b"\x02\x05hello\x06world!\x01\x02 \x00\x03\x06:::\x03\x01\x00";
let mut v = blobby::BlobIterator::new(buf).unwrap();
assert_eq!(v.next(), Some(Ok(&b"hello"[..])));
assert_eq!(v.next(), Some(Ok(&b" "[..])));
assert_eq!(v.next(), Some(Ok(&b""[..])));
assert_eq!(v.next(), Some(Ok(&b"world!"[..])));
assert_eq!(v.next(), Some(Ok(&b":::"[..])));
assert_eq!(v.next(), Some(Ok(&b"world!"[..])));
assert_eq!(v.next(), Some(Ok(&b"hello"[..])));
assert_eq!(v.next(), Some(Ok(&b""[..])));
assert_eq!(v.next(), None);

let mut v = blobby::Blob2Iterator::new(buf).unwrap();
assert_eq!(v.next(), Some(Ok([&b"hello"[..], b" "])));
assert_eq!(v.next(), Some(Ok([&b""[..], b"world!"])));
assert_eq!(v.next(), Some(Ok([&b":::"[..], b"world!"])));
assert_eq!(v.next(), Some(Ok([&b"hello"[..], b""])));
assert_eq!(v.next(), None);

let mut v = blobby::Blob4Iterator::new(buf).unwrap();
assert_eq!(v.next(), Some(Ok([&b"hello"[..], b" ", b"", b"world!"])));
assert_eq!(v.next(), Some(Ok([&b":::"[..], b"world!", b"hello", b""])));
assert_eq!(v.next(), None);
```

## Encoding and decoding

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
cargo run --releae --bin encode -- /path/to/input.txt /path/to/output.blb
```

This will create a file which can be read using `blobby::Blob2Iterator`.

To see contents of an existing Blobby file you can use the following command:
```sh
cargo run --releae --bin decode -- /path/to/input.blb /path/to/output.txt
```
The output file will contain a sequence of hex-encoded byte strings stored
in the input file. 

## Storage format

Storage format represents a sequence of binary blobs. The format uses
git-flavored [variable-length quantity][0] (VLQ) for encoding unsigned
numbers.

File starts with a number of de-duplicated blobs `d`. It followed by `d`
entries. Each entry starts with an integer `m`, immediately folowed by `m`
bytes representing de-duplicated binary blob.

Next follows unspecified number of entries representing sequence of stored
blobs. Each entry starts with an unsigned integer `n`. The least significant
bit of this integer is used as a flag. If the flag is equal to 0, then the
number is followed by `n >> 1` bytes, representing a stored binary blob.
Otherwise the entry references a de-duplicated entry number `n >> 1`.

[0]: https://en.wikipedia.org/wiki/Variable-length_quantity

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
