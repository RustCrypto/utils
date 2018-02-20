#![no_std]
#[macro_use]
extern crate arrayref;
extern crate byte_tools;
extern crate block_padding;

use byte_tools::{zero, write_u64_le, write_u64_be};
use block_padding::{Padding, PadError};

macro_rules! impl_buffer {
    ($name:ident, $len:expr) => {

        pub struct $name {
            buffer: [u8; $len],
            pos: usize,
        }

        impl Copy for $name {}

        impl Clone for $name {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name {buffer: [0; $len], pos: 0}
            }
        }

        impl $name {
            #[inline]
            pub fn input<F>(&mut self, mut input: &[u8], mut func: F)
                where F: FnMut(&[u8; $len])
            {
                // If there is already data in the buffer, process it if we have
                // enough to complete the chunk.
                let rem = self.remaining();
                if self.pos != 0 && input.len() >= rem {
                    let (l, r) = input.split_at(rem);
                    input = r;
                    self.buffer[self.pos..].copy_from_slice(l);
                    self.pos = 0;
                    func(&self.buffer);
                }

                // While we have at least a full buffer size chunks's worth of data,
                // process that data without copying it into the buffer
                while input.len() >= self.size() {
                    let (l, r) = input.split_at(self.size());
                    input = r;
                    func(array_ref!(l, 0, $len));
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
            pub fn input_lazy<F>(&mut self, mut input: &[u8], mut func: F)
                where F: FnMut(&[u8; $len])
            {
                let rem = self.remaining();
                if self.pos != 0 && input.len() > rem {
                    let (l, r) = input.split_at(rem);
                    input = r;
                    self.buffer[self.pos..].copy_from_slice(l);
                    self.pos = 0;
                    func(&self.buffer);
                }

                while input.len() > self.size() {
                    let (l, r) = input.split_at(self.size());
                    input = r;
                    func(array_ref!(l, 0, $len));
                }

                self.buffer[self.pos..self.pos+input.len()].copy_from_slice(input);
                self.pos += input.len();
            }

            /// Pad buffer with `prefix` and make sure that internall buffer
            /// has at least `up_to` free bytes. All remaining bytes get
            /// zeroed-out.
            #[inline]
            fn digest_pad<F>(&mut self, prefix: u8, up_to: usize, func: &mut F)
                where F: FnMut(&[u8; $len])
            {
                if self.pos == self.size() {
                    func(&self.buffer);
                    self.pos == 0;
                }
                self.buffer[self.pos] = prefix;
                self.pos += 1;

                zero(&mut self.buffer[self.pos..]);

                if self.remaining() < up_to {
                    func(&self.buffer);
                    zero(&mut self.buffer[..self.pos]);
                }
            }

            /// Pad message with provided prefix and 64 bit message length
            /// in big-endian format
            #[inline]
            pub fn len64_padding_be<F>(
                    &mut self, prefix: u8, data_len: u64, mut func: F)
                where F: FnMut(&[u8; $len])
            {
                self.digest_pad(prefix, 8, &mut func);
                let s = self.size();
                write_u64_be(&mut self.buffer[s-8..], data_len);
                func(&self.buffer);
                self.pos = 0;
            }

            /// Pad message with provided prefix and 64 bit message length
            /// in little-endian format
            #[inline]
            pub fn len64_padding_le<F>(
                    &mut self, prefix: u8, data_len: u64, mut func: F)
                where F: FnMut(&[u8; $len])
            {
                self.digest_pad(prefix, 8, &mut func);
                let s = self.size();
                write_u64_le(&mut self.buffer[s-8..], data_len);
                func(&self.buffer);
                self.pos = 0;
            }

            /// Pad message with provided prefix and 128 bit message length
            /// in big-endian format
            #[inline]
            pub fn len128_padding_be<F>(
                    &mut self, prefix: u8, hi: u64, lo: u64, mut func: F)
                where F: FnMut(&[u8; $len])
            {
                self.digest_pad(prefix, 16, &mut func);
                let s = self.size();
                write_u64_be(&mut self.buffer[s-16..s-8], lo);
                write_u64_be(&mut self.buffer[s-8..], hi);
                func(&self.buffer);
                self.pos = 0;
            }

            /// Pad message with provided prefix and 128 bit message length
            /// in little-endian format
            #[inline]
            pub fn len128_padding_le<F>(
                    &mut self, prefix: u8, hi: u64, lo: u64, mut func: F)
                where F: FnMut(&[u8; $len])
            {
                self.digest_pad(prefix, 16, &mut func);
                let s = self.size();
                write_u64_le(&mut self.buffer[s-16..s-8], hi);
                write_u64_le(&mut self.buffer[s-8..], lo);
                func(&self.buffer);
                self.pos = 0;
            }

            /// Pad message with given padding `P`, returns `PadError` if
            /// internall buffer is full, which can only happen if `input_lazy`
            /// was used.
            #[inline]
            pub fn pad_with<P: Padding>(&mut self)
                -> Result<&mut [u8; $len], PadError>
            {
                P::pad_block(&mut self.buffer[..], self.pos)?;
                self.pos = 0;
                Ok(&mut self.buffer)
            }

            /// Return size of the internall buffer in bytes
            #[inline]
            pub fn size(&self) -> usize {
                $len
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
    }
}

impl_buffer!(BlockBuffer128, 16);
impl_buffer!(BlockBuffer256, 32);
impl_buffer!(BlockBuffer512, 64);
impl_buffer!(BlockBuffer1024, 128);

impl_buffer!(BlockBuffer576, 72);
impl_buffer!(BlockBuffer832, 104);
impl_buffer!(BlockBuffer1088, 136);
impl_buffer!(BlockBuffer1152, 144);
impl_buffer!(BlockBuffer1344, 168);
