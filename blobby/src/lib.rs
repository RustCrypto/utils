//! Iterators over a simple binary blob storage.
//!
//! # Storage format
//! Storage format represents a sequence of binary blobs. The format uses
//! git-flavored [variable-length quantity][0] (VLQ) for encoding unsigned
//! numbers.
//!
//! File starts with a number of de-duplicated blobs `d`. It followed by `d`
//! entries. Each entry starts with an integer `m`, immediately folowed by `m`
//! bytes representing de-duplicated binary blob.
//!
//! Next follows unspecified number of entries representing sequence of stored
//! blobs. Each entry starts with an unsigned integer `n`. The least significant
//! bit of this integer is used as a flag. If the flag is equal to 0, then the
//! number is followed by `n >> 1` bytes, representing a stored binary blob.
//! Otherwise the entry references a de-duplicated entry number `n >> 1`.
//!
//! # Examples
//! ```
//! let buf = b"\x02\x05hello\x06world!\x01\x02 \x00\x03\x06:::\x03\x01\x00";
//! let mut v = blobby::BlobIterator::new(buf).unwrap();
//! assert_eq!(v.next(), Some(Ok(&b"hello"[..])));
//! assert_eq!(v.next(), Some(Ok(&b" "[..])));
//! assert_eq!(v.next(), Some(Ok(&b""[..])));
//! assert_eq!(v.next(), Some(Ok(&b"world!"[..])));
//! assert_eq!(v.next(), Some(Ok(&b":::"[..])));
//! assert_eq!(v.next(), Some(Ok(&b"world!"[..])));
//! assert_eq!(v.next(), Some(Ok(&b"hello"[..])));
//! assert_eq!(v.next(), Some(Ok(&b""[..])));
//! assert_eq!(v.next(), None);
//!
//! let mut v = blobby::Blob2Iterator::new(buf).unwrap();
//! assert_eq!(v.next(), Some(Ok([&b"hello"[..], b" "])));
//! assert_eq!(v.next(), Some(Ok([&b""[..], b"world!"])));
//! assert_eq!(v.next(), Some(Ok([&b":::"[..], b"world!"])));
//! assert_eq!(v.next(), Some(Ok([&b"hello"[..], b""])));
//! assert_eq!(v.next(), None);
//!
//! let mut v = blobby::Blob4Iterator::new(buf).unwrap();
//! assert_eq!(v.next(), Some(Ok([&b"hello"[..], b" ", b"", b"world!"])));
//! assert_eq!(v.next(), Some(Ok([&b":::"[..], b"world!", b"hello", b""])));
//! assert_eq!(v.next(), None);
//! ```
//!
//! [0]: https://en.wikipedia.org/wiki/Variable-length_quantity
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/blobby/0.3.0"
)]
extern crate alloc;

use alloc::{boxed::Box, collections::BTreeMap, vec, vec::Vec};
use core::iter::Iterator;

/// Iterator over binary blobs
pub struct BlobIterator<'a> {
    data: &'a [u8],
    dedup: Box<[&'a [u8]]>,
    pos: usize,
}

/// `blobby` error type
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// Decoded VLQ number is too big
    InvalidVlq,
    /// Invalid de-duplicated blob index
    InvalidIndex,
    /// Unexpected end of data
    UnexpectedEnd,
    /// Not enough elements for `BlobNIterator`
    NotEnoughElements,
}

const NEXT_MASK: u8 = 0b1000_0000;
const VAL_MASK: u8 = 0b0111_1111;

/// Read a git-flavoured VLQ value from `&data[*pos..]`.
/// Increments `pos` to a number of read bytes.
///
/// This function returns `None` if buffer does not contain enough bytes
/// or if VLQ is bigger than 4 bytes.
///
/// See the test submodule for example values.
fn read_vlq(data: &[u8], pos: &mut usize) -> Result<usize, Error> {
    let b = data.get(*pos).ok_or(Error::UnexpectedEnd)?;
    *pos += 1;
    let mut next = b & NEXT_MASK;
    let mut val = (b & VAL_MASK) as usize;

    macro_rules! step {
        () => {
            if next == 0 {
                return Ok(val);
            }
            let b = data.get(*pos).ok_or(Error::UnexpectedEnd)?;
            *pos += 1;
            next = b & NEXT_MASK;
            let t = (b & VAL_MASK) as usize;
            val = ((val + 1) << 7) + t;
        };
    }

    step!();
    step!();
    step!();

    if next != 0 {
        return Err(Error::InvalidVlq);
    }

    Ok(val)
}

/// Write a git-flavoured VLQ value into `buf`.
///
/// Returns the slice within `buf` that holds the value.
fn encode_vlq(mut val: usize, buf: &mut [u8; 4]) -> &[u8] {
    macro_rules! step {
        ($n:expr) => {
            buf[$n] = if $n == 3 {
                (val & (VAL_MASK as usize)) as u8
            } else {
                val -= 1;
                NEXT_MASK | (val & (VAL_MASK as usize)) as u8
            };
            val >>= 7;
            if val == 0 {
                return &buf[$n..];
            }
        };
    }

    step!(3);
    step!(2);
    step!(1);
    step!(0);
    panic!("integer is too big")
}

/// Encode the given collection of binary blobs in .blb format into `writer`.
/// Returns the encoded data together with a count of the number of blobs included in the index.
///
/// The encoded file format is:
///  - count of index entries=N
///  - N x index entries, each encoded as:
///      - size L of index entry (VLQ)
///      - index blob contents (L bytes)
///  - repeating encoded blobs, each encoded as:
///      - VLQ value that is either:
///         - (J << 1) & 0x01: indicates this blob is index entry J
///         - (L << 1) & 0x00: indicates an explicit blob of len L
///      - (in the latter case) explicit blob contents (L bytes)
pub fn encode_blobs<'a, I, T: 'a>(blobs: &'a I) -> (Vec<u8>, usize)
where
    &'a I: IntoIterator<Item = &'a T>,
    T: AsRef<[u8]>,
{
    let mut idx_map = BTreeMap::new();
    blobs
        .into_iter()
        .map(|v| v.as_ref())
        .filter(|blob| !blob.is_empty())
        .for_each(|blob| {
            let v = idx_map.entry(blob.as_ref()).or_insert(0);
            *v += 1;
        });

    let mut idx: Vec<&[u8]> = idx_map
        .iter()
        .filter(|(_, &v)| v > 1)
        .map(|(&k, _)| k)
        .collect();
    idx.sort_by_key(|e| {
        let k = match e {
            [0] => 2,
            [1] => 1,
            _ => 0,
        };
        (k, idx_map.get(e).unwrap())
    });
    idx.reverse();
    let idx_len = idx.len();

    let rev_idx: BTreeMap<&[u8], usize> = idx.iter().enumerate().map(|(i, &e)| (e, i)).collect();

    let mut out_buf = Vec::new();
    let mut buf = [0u8; 4];
    out_buf.extend_from_slice(encode_vlq(idx.len(), &mut buf));
    for e in idx {
        out_buf.extend_from_slice(encode_vlq(e.len(), &mut buf));
        out_buf.extend_from_slice(e);
    }

    for blob in blobs.into_iter().map(|v| v.as_ref()) {
        if let Some(dup_pos) = rev_idx.get(blob) {
            let n = (dup_pos << 1) + 1usize;
            out_buf.extend_from_slice(encode_vlq(n, &mut buf));
        } else {
            let n = blob.len() << 1;
            out_buf.extend_from_slice(encode_vlq(n, &mut buf));
            out_buf.extend_from_slice(blob);
        }
    }

    (out_buf, idx_len)
}

impl<'a> BlobIterator<'a> {
    /// Create new `BlobIterator` for given `data`.
    pub fn new(data: &'a [u8]) -> Result<Self, Error> {
        let mut pos = 0;
        let dedup_n = read_vlq(data, &mut pos)?;

        let mut dedup: Vec<&[u8]> = vec![&[]; dedup_n];
        for entry in dedup.iter_mut() {
            let m = read_vlq(data, &mut pos).unwrap();
            *entry = &data[pos..pos + m];
            pos += m;
        }
        Ok(BlobIterator {
            data: &data[pos..],
            dedup: dedup.into_boxed_slice(),
            pos: 0,
        })
    }

    fn read(&mut self) -> Result<&'a [u8], Error> {
        let val = read_vlq(self.data, &mut self.pos).unwrap();
        // the least significant bit is used as a flag
        let is_ref = (val & 1) != 0;
        let val = val >> 1;
        if is_ref {
            if val >= self.dedup.len() {
                return Err(Error::InvalidIndex);
            }
            Ok(self.dedup[val])
        } else {
            let s = self.pos;
            self.pos += val;
            Ok(self.data.get(s..self.pos).ok_or(Error::UnexpectedEnd)?)
        }
    }

    fn error_block(&mut self) {
        self.pos = self.data.len();
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = Result<&'a [u8], Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.data.len() {
            let val = self.read();
            if val.is_err() {
                self.error_block()
            }
            Some(val)
        } else {
            None
        }
    }
}

// TODO: use const generics on stabilization
// docs are not generated due to https://github.com/rust-lang/rust/issues/52607
macro_rules! new_iter {
    ($name:ident, $n:expr) => {
        pub struct $name<'a> {
            inner: BlobIterator<'a>,
        }

        impl<'a> $name<'a> {
            pub fn new(data: &'a [u8]) -> Result<Self, Error> {
                BlobIterator::new(data).map(|inner| Self { inner })
            }
        }

        impl<'a> Iterator for $name<'a> {
            type Item = Result<[&'a [u8]; $n], Error>;

            fn next(&mut self) -> Option<Self::Item> {
                let mut res: [&[u8]; $n] = Default::default();

                for (i, v) in res.iter_mut().enumerate() {
                    *v = match self.inner.next() {
                        Some(Ok(val)) => val,
                        Some(Err(e)) => return Some(Err(e)),
                        None if i == 0 => return None,
                        None => {
                            self.inner.error_block();
                            return Some(Err(Error::NotEnoughElements));
                        }
                    };
                }
                Some(Ok(res))
            }
        }
    };
}

new_iter!(Blob2Iterator, 2);
new_iter!(Blob3Iterator, 3);
new_iter!(Blob4Iterator, 4);
new_iter!(Blob5Iterator, 5);
new_iter!(Blob6Iterator, 6);

#[cfg(test)]
mod tests {
    use super::{read_vlq, Error, NEXT_MASK, VAL_MASK};

    fn encode_vlq(mut val: usize, buf: &mut [u8; 4]) -> &[u8] {
        macro_rules! step {
            ($n:expr) => {
                buf[$n] = if $n == 3 {
                    (val & (VAL_MASK as usize)) as u8
                } else {
                    val -= 1;
                    NEXT_MASK | (val & (VAL_MASK as usize)) as u8
                };
                val >>= 7;
                if val == 0 {
                    return &buf[$n..];
                }
            };
        }

        step!(3);
        step!(2);
        step!(1);
        step!(0);
        panic!("integer is too big")
    }

    #[test]
    fn encode_decode() {
        let mut buf = [0u8; 4];
        for val in 0..=270549119 {
            let res = encode_vlq(val, &mut buf);
            let val_res = read_vlq(res, &mut 0).unwrap();
            assert_eq!(val, val_res);
        }
    }

    #[test]
    #[rustfmt::skip]
    fn test_vlq() {
        let mut pos = 0;
        let examples = [
            0b0000_0000, // 0
            0b0000_0010, // 2
            0b0111_1111, // 127
            0b1000_0000, 0b0000_0000, // 128
            0b1111_1111, 0b0111_1111, // 16511
            0b1000_0000, 0b1000_0000, 0b0000_0000, // 16512
            0b1111_1111, 0b1111_1111, 0b0111_1111, // 2113663
            0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0000, // 2113664
            0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111, // 270549119
            0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111,
        ];

        let targets = [
            (0, 1),
            (2, 1),
            (127, 1),
            (128, 2),
            (16511, 2),
            (16512, 3),
            (2113663, 3),
            (2113664, 4),
            (270549119, 4),
        ];

        let mut buf = [0u8; 4];

        for &(val, size) in targets.iter() {
            let prev_pos = pos;
            assert_eq!(read_vlq(&examples, &mut pos), Ok(val));
            assert_eq!(pos - prev_pos, size);
            assert_eq!(encode_vlq(val, &mut buf), &examples[prev_pos..pos]);
        }

        // only VLQ values of up to 4 bytes are supported
        assert_eq!(read_vlq(&examples, &mut pos), Err(Error::InvalidVlq));
        assert_eq!(pos, 25);
    }
}
