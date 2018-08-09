#![no_std]
extern crate byte_tools;
pub extern crate block_padding;
pub extern crate generic_array;

use byte_tools::{zero, write_u64_le, write_u64_be};
use block_padding::{Padding, PadError};
use generic_array::{GenericArray, ArrayLength};

/// Buffer for block processing of data
#[derive(Clone, Default)]
pub struct BlockBuffer<BlockSize: ArrayLength<u8>>  {
    buffer: GenericArray<u8, BlockSize>,
    pos: usize,
}

impl<BlockSize: ArrayLength<u8>> BlockBuffer<BlockSize> {
    /// Process data in `input` in blocks of size `BlockSize` using function `f`.
    #[inline]
    pub fn input<F>(&mut self, mut input: &[u8], mut f: F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        // If there is already data in the buffer, process it if we have
        // enough to complete the chunk.
        let rem = self.remaining();
        if self.pos != 0 && input.len() >= rem {
            let (l, r) = input.split_at(rem);
            input = r;
            self.buffer[self.pos..].copy_from_slice(l);
            self.pos = 0;
            f(&self.buffer);
        }

        // While we have at least a full buffer size chunks's worth of data,
        // process that data without copying it into the buffer
        while input.len() >= self.size() {
            let (l, r) = input.split_at(self.size());
            input = r;
            f(GenericArray::from_slice(l));
        }

        // Copy any remaining data into the buffer.
        self.buffer[self.pos..self.pos+input.len()].copy_from_slice(input);
        self.pos += input.len();
    }

    /// Variant that doesn't flush the buffer until there's additional
    /// data to be processed. Suitable for tweakable block ciphers
    /// like Threefish that need to know whether a block is the *last*
    /// data block before processing it.
    #[inline]
    pub fn input_lazy<F>(&mut self, mut input: &[u8], mut f: F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        let rem = self.remaining();
        if self.pos != 0 && input.len() > rem {
            let (l, r) = input.split_at(rem);
            input = r;
            self.buffer[self.pos..].copy_from_slice(l);
            self.pos = 0;
            f(&self.buffer);
        }

        while input.len() > self.size() {
            let (l, r) = input.split_at(self.size());
            input = r;
            f(GenericArray::from_slice(l));
        }

        self.buffer[self.pos..self.pos+input.len()].copy_from_slice(input);
        self.pos += input.len();
    }

    /// Pad buffer with `prefix` and make sure that internall buffer
    /// has at least `up_to` free bytes. All remaining bytes get
    /// zeroed-out.
    #[inline]
    fn digest_pad<F>(&mut self, prefix: u8, up_to: usize, f: &mut F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        if self.pos == self.size() {
            f(&self.buffer);
            self.pos = 0;
        }
        self.buffer[self.pos] = prefix;
        self.pos += 1;

        zero(&mut self.buffer[self.pos..]);

        if self.remaining() < up_to {
            f(&self.buffer);
            zero(&mut self.buffer[..self.pos]);
        }
    }

    /// Pad message with provided prefix and 64 bit message length
    /// in big-endian format
    #[inline]
    pub fn len64_padding_be<F>(&mut self, prefix: u8, data_len: u64, mut f: F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        self.digest_pad(prefix, 8, &mut f);
        let s = self.size();
        write_u64_be(&mut self.buffer[s-8..], data_len);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with provided prefix and 64 bit message length
    /// in little-endian format
    #[inline]
    pub fn len64_padding_le<F>(&mut self, prefix: u8, data_len: u64, mut f: F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        self.digest_pad(prefix, 8, &mut f);
        let s = self.size();
        write_u64_le(&mut self.buffer[s-8..], data_len);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with provided prefix and 128 bit message length
    /// in big-endian format
    #[inline]
    pub fn len128_padding_be<F>(&mut self, prefix: u8, hi: u64, lo: u64, mut f: F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        self.digest_pad(prefix, 16, &mut f);
        let s = self.size();
        write_u64_be(&mut self.buffer[s-16..s-8], lo);
        write_u64_be(&mut self.buffer[s-8..], hi);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with provided prefix and 128 bit message length
    /// in little-endian format
    #[inline]
    pub fn len128_padding_le<F>(&mut self, prefix: u8, hi: u64, lo: u64, mut f: F)
        where F: FnMut(&GenericArray<u8, BlockSize>)
    {
        self.digest_pad(prefix, 16, &mut f);
        let s = self.size();
        write_u64_le(&mut self.buffer[s-16..s-8], hi);
        write_u64_le(&mut self.buffer[s-8..], lo);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with given padding `P`, returns `PadError` if
    /// internall buffer is full, which can only happen if `input_lazy`
    /// was used.
    #[inline]
    pub fn pad_with<P: Padding>(&mut self)
        -> Result<&mut GenericArray<u8, BlockSize>, PadError>
    {
        P::pad_block(&mut self.buffer[..], self.pos)?;
        self.pos = 0;
        Ok(&mut self.buffer)
    }

    /// Return size of the internall buffer in bytes
    #[inline]
    pub fn size(&self) -> usize {
        BlockSize::to_usize()
    }

    /// Return current cursor position
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Return number of remaining bytes in the internall buffer
    #[inline]
    pub fn remaining(&self) -> usize {
        self.size() - self.pos
    }
}
