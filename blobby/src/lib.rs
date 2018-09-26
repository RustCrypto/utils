#![no_std]
extern crate byteorder;

use byteorder::{LE, ByteOrder};
use core::iter::{Iterator, ExactSizeIterator};

pub struct DupBlobIterator<'a> {
    index: &'a [u8],
    data: &'a [u8],
    pos: usize,
}

impl<'a> DupBlobIterator<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        if data.len() < 8 { Err("data is too small")? }
        let (magic, data) = data.split_at(4);
        if magic != b"BLBD" { Err("invalid data prefix")? }
        let (len, data) = data.split_at(4);
        let len = LE::read_u32(len) as usize;
        if data.len() < 8*len {
            Err("data is too small for provided number of items")?
        }
        let (index, data) = data.split_at(8*len);
        for chunk in index.chunks(8) {
            let start = LE::read_u32(&chunk[..4]) as usize;
            let end = LE::read_u32(&chunk[4..]) as usize;
            if start > end { Err("index: start is bigger than end")? }
            if end > data.len() {
                Err("index: end points outside of data segment")?
            }
        }
        Ok(Self { index, data, pos: 0 })
    }
}

impl<'a> Iterator for DupBlobIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.index.len()/8 { return None; }
        let n = 8*self.pos;

        // safe because we have checked self.pos earlier
        debug_assert!(self.index.get(n..n + 8).is_some());
        let (start, end) = unsafe {(
            self.index.get_unchecked(n..n + 4),
            self.index.get_unchecked(n + 4..n + 8),
        )};
        let start = LE::read_u32(start) as usize;
        let end = LE::read_u32(end) as usize;

        self.pos += 1;

        // safe because we have checked index on initialization
        debug_assert!(self.data.get(start..end).is_some());
        Some(unsafe { self.data.get_unchecked(start..end) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.index.len()/8 - self.pos;
        (n, Some(n))
    }
}

//impl<'a> FusedIterator for DupBlobIterator<'a> { }

impl<'a> ExactSizeIterator for DupBlobIterator<'a> {
    fn len(&self) -> usize { self.index.len()/8 - self.pos }
}

pub struct UniqueBlobIterator<'a> {
    index: &'a [u8],
    data: &'a [u8],
    pos: usize,
    cursor: usize,
}

impl<'a> UniqueBlobIterator<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self, &'static str> {
        if data.len() < 8 { Err("data is too small")? }
        let (magic, data) = data.split_at(4);
        if magic != b"BLBU" { Err("invalid data prefix")? }
        let (len, data) = data.split_at(4);
        let len = LE::read_u32(len) as usize;
        if data.len() < 4*len {
            Err("data is too small for provided number of items")?
        }
        let (index, data) = data.split_at(4*len);
        let sum = index.chunks(4)
            .map(LE::read_u32)
            .fold(0, |a, v| a + v as usize);
        if sum > data.len() {
            Err("index element points outside of data segment")?
        }
        Ok(Self { index, data, pos: 0, cursor: 0 })
    }
}

impl<'a> Iterator for UniqueBlobIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        if self.pos >= self.index.len()/4 { return None; }
        let n = 4*self.pos;

        // safe because we have checked self.pos earlier
        debug_assert!(self.index.get(n..n + 4).is_some());
        let len = unsafe { self.index.get_unchecked(n..n + 4) };
        let len = LE::read_u32(len) as usize;

        let start = self.cursor;
        let end = start + len;
        self.pos += 1;
        self.cursor = end;

        // safe because we have checked index on initialization
        debug_assert!(self.data.get(start..end).is_some());
        Some(unsafe { self.data.get_unchecked(start..end) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.index.len()/4 - self.pos;
        (n, Some(n))
    }
}

//impl<'a> FusedIterator for UniqueBlobIterator<'a> { }

impl<'a> ExactSizeIterator for UniqueBlobIterator<'a> {
    fn len(&self) -> usize { self.index.len()/4 - self.pos }
}
