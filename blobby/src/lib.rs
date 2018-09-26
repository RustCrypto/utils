//! This crate provides several iterators over simply binary blob storage.
//!
//! # Storage format
//! Data in the storage format starts with magical string "blobby', next is one
//! byte which denotes how many bytes are used for blob length prefixes. For
//! example "blobby1" means that 1 byte is used for length, and "blobby2",
//! "blobby4", "blobby8" mean 2, 4 and 8 bytes respectively (this number will
//! be denoted as `n`).
//!
//! After header goes a sequence of records. Each record starts with prefix of
//! `n` bytes in which record length (excluding prefix) is stored using little
//! endian format.
//!
//! # Examples
//! First line represents input binary string and second line shows blobs stored
//! in it.
//! ```text
//! "blobby1\x05hello\x01 \x00\x05world"
//! "hello", " ", "", "world"
//! ```
//! Example for `n=2`, note that length is given in little endian format:
//! ```text
//! "blobby2\x03\x00foo\x00\x00\x03\x00bar"
//! "foo", "", "bar"
//! ```
#![doc(html_logo_url =
    "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]
#![no_std]
extern crate byteorder;

use byteorder::{LE, ByteOrder};
use core::iter::Iterator;

/// Iterator over binary blobs
pub struct BlobIterator<'a> {
    data: &'a [u8],
    size: IndexSize,
}

impl<'a> BlobIterator<'a> {
    /// Create a new `BlobIterator` for given `data`.
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        if data.len() < 7 { Err("data is too small")? }
        let (header, data) = data.split_at(7);
        let size = match header {
            b"blobby1" => IndexSize::N8,
            b"blobby2" => IndexSize::N16,
            b"blobby4" => IndexSize::N32,
            b"blobby8" => IndexSize::N64,
            _ => Err("invalid data header")?,
        };
        Ok(BlobIterator { data, size })
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.data.len() == 0 { return None; }
        let (val, leftover) = self.size.read(self.data);
        self.data = leftover;
        Some(val)
    }
}

/// Iterator over binary blob pairs
pub struct Blob2Iterator<'a> {
    inner: BlobIterator<'a>,
}

impl<'a> Blob2Iterator<'a> {
    /// Create a new `Blob2Iterator` for given `data`.
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        Ok(Self { inner: BlobIterator::new(data)? })
    }
}

impl<'a> Iterator for Blob2Iterator<'a> {
    type Item = [&'a [u8]; 2];

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

/// Iterator over binary blob triples
pub struct Blob3Iterator<'a> {
    inner: BlobIterator<'a>,
}

impl<'a> Blob3Iterator<'a> {
    /// Create a new `Blob3Iterator` for given `data`.
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        Ok(Self { inner: BlobIterator::new(data)? })
    }
}

impl<'a> Iterator for Blob3Iterator<'a> {
    type Item = [&'a [u8]; 3];

    fn next(&mut self) -> Option<Self::Item> {
        let mut res = Self::Item::default();
        for (i, v) in res.iter_mut().enumerate() {
            *v = match self.inner.next() {
                Some(val) => val,
                None if i == 0 => return None,
                None => panic!("failed to get 3 blobs, not enough elements."),
            };
        }
        Some(res)
    }
}

/// Iterator over binary blob quadruples
pub struct Blob4Iterator<'a> {
    inner: BlobIterator<'a>,
}

impl<'a> Blob4Iterator<'a> {
    /// Create a new `Blob4Iterator` for given `data`.
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        Ok(Self { inner: BlobIterator::new(data)? })
    }
}

impl<'a> Iterator for Blob4Iterator<'a> {
    type Item = [&'a [u8]; 4];

    fn next(&mut self) -> Option<Self::Item> {
        let mut res = Self::Item::default();
        for (i, v) in res.iter_mut().enumerate() {
            *v = match self.inner.next() {
                Some(val) => val,
                None if i == 0 => return None,
                None => panic!("failed to get 4 blobs, not enough elements."),
            };
        }
        Some(res)
    }
}

#[derive(Copy, Clone)]
enum IndexSize {
    N8,
    N16,
    N32,
    N64,
}

macro_rules! branch_read {
    ($data:ident, $n:expr, $method:ident) => {{
        let (size, data) = $data.split_at($n);
        let n = LE::$method(size) as usize;
        data.split_at(n)
    }}
}

impl IndexSize {
    fn read<'a>(self, data: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        match self {
            IndexSize::N8 => {
                let n = data[0] as usize;
                data[1..].split_at(n)
            },
            IndexSize::N16 => branch_read!(data, 2, read_u16),
            IndexSize::N32 => branch_read!(data, 4, read_u32),
            IndexSize::N64 => branch_read!(data, 8, read_u64),
        }
    }
}
