//! Fixed size buffer for block processing of data.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms)]

pub use generic_array;

use core::{fmt, ops::Add, slice};
pub use crypto_common::Block;
use crypto_common::{BlockSizeUser, BlockSizes};
use generic_array::{
    typenum::{Add1, B1},
    ArrayLength, GenericArray,
};

mod read;
mod sealed;

pub use read::ReadBuffer;

/// Block with additional one byte
type BlockP1<BlockSize> = GenericArray<u8, Add1<BlockSize>>;

/// Trait for buffer kinds.
pub trait BufferKind: sealed::Sealed {}

/// Eager block buffer kind, which guarantees that buffer position
/// always lies in the range of `0..BlockSize`.
#[derive(Copy, Clone, Debug, Default)]
pub struct Eager {}

/// Lazy block buffer kind, which guarantees that buffer position
/// always lies in the range of `0..=BlockSize`.
#[derive(Copy, Clone, Debug, Default)]
pub struct Lazy {}

impl BufferKind for Eager {}

impl BufferKind for Lazy {}

/// Eager block buffer.
pub type EagerBuffer<B> = BlockBuffer<B, Eager>;
/// Lazy block buffer.
pub type LazyBuffer<B> = BlockBuffer<B, Lazy>;

/// Block buffer error.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("Block buffer error")
    }
}

/// Buffer for block processing of data.
#[derive(Debug)]
pub struct BlockBuffer<BS: BlockSizes, K: BufferKind> {
    buffer: Block<Self>,
    pos: K::Pos,
}

impl<BS: BlockSizes, K: BufferKind> BlockSizeUser for BlockBuffer<BS, K> {
    type BlockSize = BS;
}

impl<BS: BlockSizes, K: BufferKind> Default for BlockBuffer<BS, K> {
    #[inline]
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            pos: Default::default(),
        }
    }
}

impl<BS: BlockSizes, K: BufferKind> Clone for BlockBuffer<BS, K> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            pos: self.pos.clone(),
        }
    }
}

impl<BS: BlockSizes, K: BufferKind> BlockBuffer<BS, K> {
    /// Create new buffer from slice.
    ///
    /// # Panics
    /// If slice length is not valid for used buffer kind.
    #[inline(always)]
    pub fn new(buf: &[u8]) -> Self {
        Self::try_new(buf).unwrap()
    }

    /// Create new buffer from slice.
    ///
    /// Returns an error if slice length is not valid for used buffer kind.
    #[inline(always)]
    pub fn try_new(buf: &[u8]) -> Result<Self, Error> {
        let pos = buf.len();
        if !K::invariant(pos, BS::USIZE) {
            return Err(Error);
        }
        let mut res = Self::default();
        res.buffer[..pos].copy_from_slice(buf);
        K::set_pos(&mut res.buffer, &mut res.pos, pos);
        Ok(res)
    }

    /// Digest data in `input` in blocks of size `BlockSize` using
    /// the `compress` function, which accepts slice of blocks.
    #[inline]
    pub fn digest_blocks(&mut self, mut input: &[u8], mut compress: impl FnMut(&[Block<Self>])) {
        let pos = self.get_pos();
        // using `self.remaining()` for some reason
        // prevents panic elimination
        let rem = self.size() - pos;
        let n = input.len();
        // Note that checking condition `pos + n < BlockSize` is
        // equivalent to checking `n < rem`, where `rem` is equal
        // to `BlockSize - pos`. Using the latter allows us to work
        // around compiler accounting for possible overflow of
        // `pos + n` which results in it inserting unreachable
        // panic branches. Using `unreachable_unchecked` in `get_pos`
        // we convince compiler that `BlockSize - pos` never underflows.
        if K::invariant(n, rem) {
            // double slicing allows to remove panic branches
            self.buffer[pos..][..n].copy_from_slice(input);
            self.set_pos_unchecked(pos + n);
            return;
        }
        if pos != 0 {
            let (left, right) = input.split_at(rem);
            input = right;
            self.buffer[pos..].copy_from_slice(left);
            compress(slice::from_ref(&self.buffer));
        }

        let (blocks, leftover) = K::split_blocks(input);
        if !blocks.is_empty() {
            compress(blocks);
        }

        let n = leftover.len();
        self.buffer[..n].copy_from_slice(leftover);
        self.set_pos_unchecked(n);
    }

    /// Reset buffer by setting cursor position to zero.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.set_pos_unchecked(0);
    }

    /// Pad remaining data with zeros and return resulting block.
    #[inline(always)]
    pub fn pad_with_zeros(&mut self) -> Block<Self> {
        let pos = self.get_pos();
        let mut res = self.buffer.clone();
        res[pos..].iter_mut().for_each(|b| *b = 0);
        self.set_pos_unchecked(0);
        res
    }

    /// Return current cursor position.
    #[inline(always)]
    pub fn get_pos(&self) -> usize {
        let pos = K::get_pos(&self.buffer, &self.pos);
        if !K::invariant(pos, BS::USIZE) {
            debug_assert!(false);
            // SAFETY: `pos` never breaks the invariant
            unsafe {
                core::hint::unreachable_unchecked();
            }
        }
        pos
    }

    /// Return slice of data stored inside the buffer.
    #[inline(always)]
    pub fn get_data(&self) -> &[u8] {
        &self.buffer[..self.get_pos()]
    }

    /// Set buffer content and cursor position.
    ///
    /// # Panics
    /// If `pos` is bigger or equal to block size.
    #[inline]
    pub fn set(&mut self, buf: Block<Self>, pos: usize) {
        assert!(K::invariant(pos, BS::USIZE));
        self.buffer = buf;
        self.set_pos_unchecked(pos);
    }

    /// Return size of the internal buffer in bytes.
    #[inline(always)]
    pub fn size(&self) -> usize {
        BS::USIZE
    }

    /// Return number of remaining bytes in the internal buffer.
    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.size() - self.get_pos()
    }

    #[inline(always)]
    fn set_pos_unchecked(&mut self, pos: usize) {
        debug_assert!(K::invariant(pos, BS::USIZE));
        K::set_pos(&mut self.buffer, &mut self.pos, pos)
    }
}

impl<BS: BlockSizes> BlockBuffer<BS, Eager> {
    /// Compress remaining data after padding it with `delim`, zeros and
    /// the `suffix` bytes. If there is not enough unused space, `compress`
    /// will be called twice.
    ///
    /// # Panics
    /// If suffix length is bigger than block size.
    #[inline(always)]
    pub fn digest_pad(&mut self, delim: u8, suffix: &[u8], mut compress: impl FnMut(&Block<Self>)) {
        if suffix.len() > BS::USIZE {
            panic!("suffix is too long");
        }
        let pos = self.get_pos();
        self.buffer[pos] = delim;
        for b in &mut self.buffer[pos + 1..] {
            *b = 0;
        }

        let n = self.size() - suffix.len();
        if self.size() - pos - 1 < suffix.len() {
            compress(&self.buffer);
            let mut block = Block::<Self>::default();
            block[n..].copy_from_slice(suffix);
            compress(&block);
        } else {
            self.buffer[n..].copy_from_slice(suffix);
            compress(&self.buffer);
        }
        self.set_pos_unchecked(0)
    }

    /// Pad message with 0x80, zeros and 64-bit message length using
    /// big-endian byte order.
    #[inline]
    pub fn len64_padding_be(&mut self, data_len: u64, compress: impl FnMut(&Block<Self>)) {
        self.digest_pad(0x80, &data_len.to_be_bytes(), compress);
    }

    /// Pad message with 0x80, zeros and 64-bit message length using
    /// little-endian byte order.
    #[inline]
    pub fn len64_padding_le(&mut self, data_len: u64, compress: impl FnMut(&Block<Self>)) {
        self.digest_pad(0x80, &data_len.to_le_bytes(), compress);
    }

    /// Pad message with 0x80, zeros and 128-bit message length using
    /// big-endian byte order.
    #[inline]
    pub fn len128_padding_be(&mut self, data_len: u128, compress: impl FnMut(&Block<Self>)) {
        self.digest_pad(0x80, &data_len.to_be_bytes(), compress);
    }

    /// Serialize buffer into a byte array.
    #[inline]
    pub fn serialize(&self) -> Block<Self> {
        let mut res = self.buffer.clone();
        let pos = self.get_pos();
        // zeroize "garbage" data
        for b in res[pos..BS::USIZE - 1].iter_mut() {
            *b = 0;
        }
        res
    }

    /// Deserialize buffer from a byte array.
    #[inline]
    pub fn deserialize(buffer: &Block<Self>) -> Result<Self, Error> {
        let pos = buffer[BS::USIZE - 1] as usize;
        if !<Eager as sealed::Sealed>::invariant(pos, BS::USIZE) {
            return Err(Error);
        }
        if buffer[pos..BS::USIZE - 1].iter().any(|&b| b != 0) {
            return Err(Error);
        }
        Ok(Self {
            buffer: buffer.clone(),
            pos: Default::default(),
        })
    }
}

impl<BS: BlockSizes> BlockBuffer<BS, Lazy> {
    /// Serialize buffer into a byte array.
    #[inline]
    pub fn serialize(&self) -> BlockP1<BS>
    where
        BS: Add<B1>,
        Add1<BS>: ArrayLength<u8>,
    {
        let mut res = BlockP1::<BS>::default();
        res[0] = self.pos;
        let data = self.get_data();
        res[1..][..data.len()].copy_from_slice(data);
        res
    }

    /// Deserialize buffer from a byte array.
    #[inline]
    pub fn deserialize(buffer: &BlockP1<BS>) -> Result<Self, Error>
    where
        BS: Add<B1>,
        Add1<BS>: ArrayLength<u8>,
    {
        let pos = buffer[0];
        if !<Lazy as sealed::Sealed>::invariant(pos as usize, BS::USIZE) {
            return Err(Error);
        }
        if buffer[1..][pos as usize..].iter().any(|&b| b != 0) {
            return Err(Error);
        }
        Ok(Self {
            buffer: GenericArray::clone_from_slice(&buffer[1..]),
            pos,
        })
    }
}
