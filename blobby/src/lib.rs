#![no_std]
extern crate byteorder;

use byteorder::{LE, ByteOrder};
use core::iter::{Iterator, FusedIterator, ExactSizeIterator};

pub struct BlobIterator<'a> {
    index: &'a [u8],
    data: &'a [u8],
    pos: usize,
}

impl<'a> BlobIterator<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        let (magic, data) = data.split_at(6);
        if magic != b"blobby" { Err("invalid data prefix")? }
        let (len, data) = data.split_at(2);
        let len = LE::read_u16(len) as usize;
        let (index, data) = data.split_at(8*len);
        for chunk in index.chunks(8) {
            let start = LE::read_u32(&chunk[..4]) as usize;
            let end = LE::read_u32(&chunk[4..]) as usize;
            if start > end { Err("index: start is bigger than end")? }
            if end > data.len() {
                Err("index: end points outside of data index")?
            }
        }
        Ok(Self { index, data, pos: 0 })
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.index.len()/8 { return None; }
        let n = 8*self.pos;
        let start = LE::read_u32(&self.index[n..n + 4]) as usize;
        let end = LE::read_u32(&self.index[n + 4..n + 8]) as usize;
        self.pos += 1;
        Some(&self.data[start..end])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.index.len()/8 - self.pos;
        (n, Some(n))
    }
}

impl<'a> FusedIterator for BlobIterator<'a> { }

impl<'a> ExactSizeIterator for BlobIterator<'a> {
    fn len(&self) -> usize { self.index.len()/8 - self.pos }
}
