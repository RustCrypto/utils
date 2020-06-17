//! Iterators over a simple binary blob storage.
//!
//! # Storage format
//! Storage format represents a sequence of binary blobs. Each entry starts
//! with an unsigned integer `n` encoded using [variable-length quantity][0].
//! The least significant bit of this integer is used as a flag. If the flag
//! is equal to 0, then the number is followed by `n >> 1` bytes, representing
//! a stored binary blob. Otherwise the entry references an entry stored
//! `n >> 1` bytes before the current position (after reading the integer).
//! Reference entries can not reference other reference entries.
//!
//! # Examples
//! ```
//! // 0x0C = 5 << 1; 0x02 = 1 << 1
//! // 0x29 = (20 << 1) + 1 -- represents offset of minus 20 bytes
//! let buf = b"\x0C hello\x02 \x00\x0C world\x02,\x29";
//! let mut v = blobby::BlobIterator::new(buf);
//! assert_eq!(v.next().unwrap(), b" hello");
//! assert_eq!(v.next().unwrap(), b" ");
//! assert_eq!(v.next().unwrap(), b"");
//! assert_eq!(v.next().unwrap(), b" world");
//! assert_eq!(v.next().unwrap(), b",");
//! assert_eq!(v.next().unwrap(), b" hello");
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
    pos: usize,
}

/// Read a git-flavoured VLQ value from `&data[*pos..]`.
/// Increments `pos` to a number of read bytes.
///
/// This function returns `None` if buffer does not contain enough bytes
/// or if VLQ is bigger than 4 bytes.
///
/// See the test submodule for example values.
fn read_vlq(data: &[u8], pos: &mut usize) -> Option<usize> {
    const NEXT_MASK: u8 = 0b1000_0000;
    const VAL_MASK: u8 = 0b0111_1111;

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
        BlobIterator { data, pos: 0 }
    }

    fn read(&mut self, second: bool) -> &'a [u8] {
        let val = read_vlq(self.data, &mut self.pos).unwrap();
        // the least significant bit is used as a flag
        let is_ref = (val & 1) != 0;
        let val = val >> 1;
        if is_ref {
            assert!(self.pos >= val);
            // prevenets potential infinite recursion
            assert!(!second);
            let t = self.pos;
            self.pos -= val;
            let buf = self.read(true);
            self.pos = t;
            buf
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
            Some(self.read(false))
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
    #[test]
    fn test_vlq() {
        use super::read_vlq;

        let mut pos = 0;
        let examples = [
            0b0000_0000, // 0
            0b0000_0010, // 2
            0b0111_1111, // 127
            0b1000_0000, 0b0000_0000, // 128
            0b1111_1111, 0b0111_1111, // 4223
            0b1000_0000, 0b1000_0000, 0b0000_0000, // 4224
            0b1111_1111, 0b1111_1111, 0b0111_1111, // 2113663
            0b1000_0000, 0b1000_0000, 0b1000_0000, 0b0000_0000, // 2113664
            0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111, // 270549119
            0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0111_1111,
        ];

        assert_eq!(read_vlq(&examples, &mut pos), Some(0));
        assert_eq!(pos, 1);

        assert_eq!(read_vlq(&examples, &mut pos), Some(2));
        assert_eq!(pos, 2);

        assert_eq!(read_vlq(&examples, &mut pos), Some(127));
        assert_eq!(pos, 3);

        assert_eq!(read_vlq(&examples, &mut pos), Some(128));
        assert_eq!(pos, 5);

        assert_eq!(read_vlq(&examples, &mut pos), Some(16511));
        assert_eq!(pos, 7);

        assert_eq!(read_vlq(&examples, &mut pos), Some(16512));
        assert_eq!(pos, 10);

        assert_eq!(read_vlq(&examples, &mut pos), Some(2113663));
        assert_eq!(pos, 13);

        assert_eq!(read_vlq(&examples, &mut pos), Some(2113664));
        assert_eq!(pos, 17);

        assert_eq!(read_vlq(&examples, &mut pos), Some(270549119));
        assert_eq!(pos, 21);

        // only VLQ values of up to 4 bytes are supported
        assert_eq!(read_vlq(&examples, &mut pos), None);
        assert_eq!(pos, 25);
    }
}
