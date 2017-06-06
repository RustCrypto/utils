#![no_std]
extern crate generic_array;
extern crate byte_tools;
use generic_array::{GenericArray, ArrayLength};
use byte_tools::{zero, write_u64_le};

type Block<N> = GenericArray<u8, N>;

#[derive(Default, Clone, Copy)]
pub struct BlockBuffer<N: ArrayLength<u8>> where N::ArrayType: Copy {
    pub buffer: GenericArray<u8, N>,
    pub pos: usize,
}

impl <N: ArrayLength<u8>> BlockBuffer<N> where N::ArrayType: Copy {
    #[inline]
    pub fn input<F: FnMut(&Block<N>)>(&mut self, mut input: &[u8], mut func: F) {
        // If there is already data in the buffer, copy as much as we can
        // into it and process the data if the buffer becomes full.
        if self.pos != 0 {
            let rem = self.remaining();

            if input.len() >= rem {
                let (l, r) = input.split_at(rem);
                input = r;
                self.buffer[self.pos..].copy_from_slice(l);
                self.pos = 0;
                func(&self.buffer);
            } else {
                let end = self.pos + input.len();
                self.buffer[self.pos..end].copy_from_slice(input);
                self.pos = end;
                return;
            }
        }

        // While we have at least a full buffer size chunks's worth of data,
        // process that data without copying it into the buffer
        while input.len() >= self.size() {
            let (l, r) = input.split_at(self.size());
            input = r;
            func(GenericArray::from_slice(&l));
        }

        // Copy any input data into the buffer. At this point in the method,
        // the ammount of data left in the input vector will be less than
        // the buffer size and the buffer will be empty.
        self.buffer[..input.len()].copy_from_slice(input);
        self.pos = input.len();
    }

    #[inline]
    fn digest_pad<F>(&mut self, up_to: usize, mut func: &mut F)
        where F: FnMut(&Block<N>)
    {
        self.buffer[self.pos] = 0x80;
        self.pos += 1;

        zero(&mut self.buffer[self.pos..]);

        if self.remaining() < up_to {
            func(&self.buffer);
            zero(&mut self.buffer[..self.pos]);
        }
    }

    #[inline]
    /// Will pad message with message length in little-endian format
    pub fn len_padding<F>(&mut self, data_len: u64, mut func: F)
        where F: FnMut(&Block<N>)
    {
        self.digest_pad(8, &mut func);
        let s = self.size();
        write_u64_le(&mut self.buffer[s-8..], data_len);
        func(&self.buffer);
        self.pos = 0;
    }

    #[inline]
    pub fn len_padding_u128<F>(&mut self, hi: u64, lo: u64, mut func: F)
        where F: FnMut(&Block<N>)
    {
        self.digest_pad(16, &mut func);
        let s = self.size();
        write_u64_le(&mut self.buffer[s-16..s-8], hi);
        write_u64_le(&mut self.buffer[s-8..], lo);
        func(&self.buffer);
        self.pos = 0;
    }

    #[inline]
    pub fn size(&self) -> usize {
         N::to_usize()
    }

    #[inline]
    pub fn position(&self) -> usize {
         self.pos
    }

    #[inline]
    pub fn remaining(&self) -> usize {
         N::to_usize() - self.pos
    }
}

/*
pub trait Padding {
    fn pad<F, N>(self, buf: BlockBuffer<N>, func: F)
        where F: FnMut(&Block<N>), N: ArrayLength<u8>, N::ArrayType: Copy;
}

#[derive(Copy, Clone, Debug)]
pub struct UnpadError;

pub trait Unpadding {
    fn unpad(self, data: &[u8]) -> Result<&[u8], UnpadError>;
}

#[derive(Default, Copy, Clone, Debug)]
struct ZeroPadding;

impl Padding for ZeroPadding {
    fn pad<F, N>(self, mut buf: BlockBuffer<N>, mut func: F)
        where F: FnMut(&Block<N>), N: ArrayLength<u8>, N::ArrayType: Copy
    {
        zero(&mut buf.buffer[buf.pos..])
        func(&buf.buffer);
    }
}

impl Unpadding for ZeroPadding {
    fn unpad(self, data: &[u8]) -> Result<&[u8], UnpadError> {
        let mut n = data.len() - 1;
        while n != 0 {
            if data[n] != 0 {
                break;
            }
            n -= 1;
        }
        Ok(&data[..n+1])
    }
}

#[derive(Default, Copy, Clone, Debug)]
struct AnsiX923;

impl Padding for AnsiX923 {
    fn pad<F, N>(self, mut buf: BlockBuffer<N>, mut func: F)
        where F: FnMut(&Block<N>), N: ArrayLength<u8>, N::ArrayType: Copy
    {
        let n = N::to_usize() - 1;
        for b in buf.buffer[buf.pos..n].iter_mut() {
            *b = 0;
        }
        buf.buffer[n] = (n - buf.pos) as u8;
        func(&buf.buffer);
    }
}

impl Unpadding for AnsiX923 {
    fn unpad(self, data: &[u8]) -> Result<&[u8], UnpadError> {
        if data.is_empty() { return Err(UnpadError); }
        let l = data.len();
        let n = data[l-1] as usize;
        if n == 0 {
            return Err(UnpadError)
        }
        for v in &data[l-n..l-1] {
            if *v != 0 { return Err(UnpadError); }
        }
        Ok(&data[..l-n])
    }
}

#[derive(Default, Copy, Clone, Debug)]
struct Pkcs7;

impl Padding for Pkcs7 {
    fn pad<F, N>(self, mut buf: BlockBuffer<N>, mut func: F)
        where F: FnMut(&Block<N>), N: ArrayLength<u8>, N::ArrayType: Copy
    {
        let n = N::to_usize() - buf.pos;
        for b in buf.buffer[buf.pos..].iter_mut() {
            *b = n as u8;
        }
        func(&buf.buffer);
    }
}

impl Unpadding for Pkcs7 {
    fn unpad(self, data: &[u8]) -> Result<&[u8], UnpadError> {
        if data.is_empty() { return Err(UnpadError); }
        let l = data.len();
        let n = data[l-1];
        if n == 0 {
            return Err(UnpadError)
        }
        for v in &data[l-n as usize..l-1] {
            if *v != n { return Err(UnpadError); }
        }
        Ok(&data[..l-n as usize])
    }
}

#[derive(Default, Copy, Clone, Debug)]
struct Iso7816;

impl Padding for Iso7816 {
    fn pad<F, N>(self, mut buf: BlockBuffer<N>, mut func: F)
        where F: FnMut(&Block<N>), N: ArrayLength<u8>, N::ArrayType: Copy
    {
        let n = N::to_usize() - buf.pos;
        for b in buf.buffer[buf.pos+1..].iter_mut() {
            *b = n as u8;
        }
        buf.buffer[buf.pos] = 0x80;
        func(&buf.buffer);
    }
}

impl Unpadding for Iso7816 {
    fn unpad(self, data: &[u8]) -> Result<&[u8], UnpadError> {
        if data.is_empty() { return Err(UnpadError); }
        let mut n = data.len() - 1;
        while n != 0 {
            if data[n] != 0 {
                break;
            }
            n -= 1;
        }
        if data[n] != 0x80 { return Err(UnpadError); }
        Ok(&data[..n])
    }
}
*/