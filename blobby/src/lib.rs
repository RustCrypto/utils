#![no_std]
extern crate byteorder;

use byteorder::{LE, ByteOrder};
use core::iter::Iterator;

pub struct BlobIterator<'a> {
    index: &'a [u8],
    data: &'a [u8],
    n: usize,
    cursor: usize,
}

impl<'a> BlobIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        let (len, data) = data.split_at(4);
        let len = LE::read_u32(len) as usize;
        let (index, data) = data.split_at(4*len);
        let sum = index.chunks(4)
            .map(|chunk| LE::read_u32(chunk) as usize)
            .fold(0, |acc, x| acc + x);
        assert!(sum <= data.len());
        Self { index, data, n: 0, cursor: 0 }
    }
}

impl<'a> Iterator for BlobIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        let n = 4*self.n;
        if n >= self.index.len() { return None; }
        let len = LE::read_u32(&self.index[n..n + 4]) as usize;
        let cursor = self.cursor;
        self.cursor += len;
        self.n += 1;
        Some(&self.data[cursor..cursor + len])
    }
}
