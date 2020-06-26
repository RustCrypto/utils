//! Iterators over a simple binary blob storage.
//!
//! # Storage format
//! Storage format represents a sequence of binary blobs. The format uses
//! git-flavored [variable-length quantity][0] (VLQ) for encoding unsigned
//! numbers.
//!
//! File starts with a number of de-duplicated blobs `d`. It followed by `d`
//! entries. Each entry starts with a integer `m`, immediately folowed by `m`
//! bytes representing de-duplicated binary blob.
//!
//! Next follows unspecified number of entries representing sequence of stored
//! blobs. Each entry starts with an unsigned integer `n`. The least significant
//! bit of this integer is used as a flag. If the flag is equal to 0, then the
//! number is followed by `n >> 1` bytes, representing a stored binary blob.
//! Otherwise the entry references a de-duplicated entry number `n >> 1` stored
//! at the beginning of the storage.
//!
//! # Examples
//! ```
//! let buf = b"\x02\x05hello\x06world!\x01\x02 \x00\x03\x06:::\x03\x01";
//! let mut v = blobby::BlobIterator::new(buf);
//! assert_eq!(v.next().unwrap(), b"hello");
//! assert_eq!(v.next().unwrap(), b" ");
//! assert_eq!(v.next().unwrap(), b"");
//! assert_eq!(v.next().unwrap(), b"world!");
//! assert_eq!(v.next().unwrap(), b":::");
//! assert_eq!(v.next().unwrap(), b"world!");
//! assert_eq!(v.next().unwrap(), b"hello");
//! assert_eq!(v.next(), None);
//! ```
//! [0]: https://en.wikipedia.org/wiki/Variable-length_quantity
#![no_std]
#![doc(html_logo_url =
    "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]

use core::iter::Iterator;

/// Iterator over binary blobs
pub struct BlobIterator<'a> {
    data: &'a [u8],
    idx_n: usize,
    pos: usize,
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
fn read_vlq(data: &[u8], pos: &mut usize) -> Option<usize> {
    let b = data.get(*pos)?;
    *pos += 1;
    let mut next = b & NEXT_MASK;
    let mut val = (b & VAL_MASK) as usize;

    macro_rules! step {
        () => {
            if next == 0 {
                return Some(val)
            }
            let b = data.get(*pos)?;
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
        return None;
    }

    Some(val)
}

impl<'a> BlobIterator<'a> {
    /// Create a new `BlobIterator` for given `data`.
    pub fn new(data: &'a [u8]) -> Self {
        let mut pos = 0;
        let idx_n = read_vlq(data, &mut pos).unwrap();
        let data = &data[pos..];
        pos = 0;
        for _ in 0..idx_n {
            let m = read_vlq(data, &mut pos).unwrap();
            pos += m;
        }
        BlobIterator { data, idx_n, pos }
    }

    fn get_idx_entry(&self, n: usize) -> &'a [u8] {
        // not the most efficient approach... an alternative would be
        // to cache index entry positions on a heap
        let mut pos = 0;
        for _ in 0..n {
            let m = read_vlq(self.data, &mut pos).unwrap();
            pos += m;
        }
        let m = read_vlq(self.data, &mut pos).unwrap();
        &self.data[pos..pos+m]
    }

    fn read(&mut self) -> &'a [u8] {
        let val = read_vlq(self.data, &mut self.pos).unwrap();
        // the least significant bit is used as a flag
        let is_ref = (val & 1) != 0;
        let val = val >> 1;
        if is_ref {
            assert!(val < self.idx_n);
            self.get_idx_entry(val)
        } else {
            let s = self.pos;
            self.pos += val;
            &self.data[s..self.pos]
        }
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.pos < self.data.len() {
            Some(self.read())
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
            pub fn new(data: &'a [u8]) -> Self {
                Self { inner: BlobIterator::new(data) }
            }
        }

        impl<'a> Iterator for $name<'a> {
            type Item = [&'a [u8]; $n];

            fn next(&mut self) -> Option<Self::Item> {
                let mut res = Self::Item::default();
                for (i, v) in res.iter_mut().enumerate() {
                    *v = match self.inner.next() {
                        Some(val) => val,
                        None if i == 0 => return None,
                        None => panic!("failed to get 2 blobs, not enough elements."),
                    };
                }
                Some(res)
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
    use super::{read_vlq, VAL_MASK, NEXT_MASK};

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
            assert_eq!(read_vlq(&examples, &mut pos), Some(val));
            assert_eq!(pos - prev_pos, size);
            assert_eq!(encode_vlq(val, &mut buf), &examples[prev_pos..pos]);
        }

        // only VLQ values of up to 4 bytes are supported
        assert_eq!(read_vlq(&examples, &mut pos), None);
        assert_eq!(pos, 25);
    }
}
