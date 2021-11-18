//! Fixed size buffer for block processing of data.
#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/block-buffer/0.10.0"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "block-padding")]
pub use block_padding;
pub use generic_array;

use core::{slice, marker::PhantomData};
use generic_array::{GenericArray, ArrayLength};
#[cfg(feature = "block-padding")]
use block_padding::{PadError, Padding};

mod sealed;

/// Block on which `BlockBuffer` operates.
pub type Block<BlockSize> = GenericArray<u8, BlockSize>;

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

/// Buffer for block processing of data.
pub struct BlockBuffer<BlockSize: ArrayLength<u8>, K: BufferKind> {
    buffer: Block<BlockSize>,
    pos: usize,
    _pd: PhantomData<K>,
}

impl<B: ArrayLength<u8>, K: BufferKind> Default for BlockBuffer<B, K> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            pos: 0,
            _pd: PhantomData,
        }
    }
}

impl<B: ArrayLength<u8>, K: BufferKind> Clone for BlockBuffer<B, K> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            pos: self.pos,
            _pd: PhantomData,
        }
    }
}

impl<BlockSize: ArrayLength<u8>, K: BufferKind> BlockBuffer<BlockSize, K> {
    /// Create new buffer from slice.
    ///
    /// # Panics
    /// If slice length is not valid for used buffer kind.
    #[inline(always)]
    pub fn new(buf: &[u8]) -> Self {
        let pos = buf.len();
        K::assert_pos(pos, BlockSize::USIZE);
        let mut buffer = Block::<BlockSize>::default();
        buffer[..pos].copy_from_slice(buf);
        let _pd = PhantomData;
        Self { buffer, pos, _pd }
    }

    /// Digest data in `input` in blocks of size `BlockSize` using
    /// the `compress` function, which accepts slice of blocks.
    #[inline]
    pub fn digest_blocks(&mut self, mut input: &[u8], mut compress: impl FnMut(&[Block<BlockSize>])) {
        let pos = self.get_pos();
        let r = self.remaining();
        let n = input.len();
        if K::input_check(n, r) {
            // double slicing allows to remove panic branches
            self.buffer[pos..][..n].copy_from_slice(input);
            self.set_pos_unchecked(pos + n);
            return;
        }
        if pos != 0 {
            let (left, right) = input.split_at(r);
            input = right;
            self.buffer[pos..].copy_from_slice(left);
            compress(slice::from_ref(&self.buffer));
        }

        let (blocks, leftover) = K::split_blocks(input);
        compress(blocks);

        let n = leftover.len();
        self.buffer[..n].copy_from_slice(leftover);
        self.set_pos_unchecked(n);
    }

    /// Reset buffer by setting cursor position to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Pad remaining data with zeros and return resulting block.
    pub fn pad_with_zeros(&mut self) -> &mut Block<BlockSize> {
        let pos = self.get_pos();
        self.buffer[pos..].iter_mut().for_each(|b| *b = 0);
        self.set_pos_unchecked(0);
        &mut self.buffer
    }

    /// Return current cursor position.
    #[inline]
    pub fn get_pos(&self) -> usize {
        let pos = self.pos;
        // SAFETY: `pos` is set only to correct values for given `K`
        unsafe {
            K::unreachable_hint(pos, BlockSize::USIZE);
        }
        pos
    }

    /// Set buffer content and cursor position.
    ///
    /// # Panics
    /// If `pos` is bigger or equal to block size.
    pub fn set(&mut self, buf: Block<BlockSize>, pos: usize) {
        K::assert_pos(pos, BlockSize::USIZE);
        self.buffer = buf;
        self.pos = pos;
    }

    /// Return size of the internall buffer in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        BlockSize::USIZE
    }

    /// Return number of remaining bytes in the internall buffer.
    #[inline]
    pub fn remaining(&self) -> usize {
        self.size() - self.get_pos()
    }

    #[inline]
    fn set_pos_unchecked(&mut self, pos: usize) {
        K::debug_assert_pos(pos, BlockSize::USIZE);
        self.pos = pos;
    }
}

impl<BlockSize: ArrayLength<u8>> BlockBuffer<BlockSize, Eager> {
    /// Set `data` to generated blocks.
    #[inline]
    pub fn set_data(
        &mut self,
        mut data: &mut [u8],
        mut process_blocks: impl FnMut(&mut [Block<BlockSize>]),
    ) {
        let pos = self.get_pos();
        let r = self.remaining();
        let n = data.len();
        if pos != 0 {
            if n < r {
                // double slicing allows to remove panic branches
                data.copy_from_slice(&self.buffer[pos..][..n]);
                self.set_pos_unchecked(pos + n);
                return;
            }
            let (left, right) = data.split_at_mut(r);
            data = right;
            left.copy_from_slice(&self.buffer[pos..]);
        }

        let (blocks, leftover) = to_blocks_mut(data);
        process_blocks(blocks);

        let n = leftover.len();
        if n != 0 {
            let mut block = Default::default();
            process_blocks(slice::from_mut(&mut block));
            leftover.copy_from_slice(&block[..n]);
            self.buffer = block;
        }
        self.set_pos_unchecked(n);
    }

    /// Compress remaining data after padding it with `delim`, zeros and
    /// the `suffix` bytes. If there is not enough unused space, `compress`
    /// will be called twice.
    #[inline(always)]
    pub fn digest_pad(
        &mut self,
        delim: u8,
        suffix: &[u8],
        mut compress: impl FnMut(&Block<BlockSize>),
    ) {
        let pos = self.get_pos();
        self.buffer[pos] = delim;
        for b in &mut self.buffer[pos + 1..] {
            *b = 0;
        }

        let n = self.size() - suffix.len();
        if self.size() - pos - 1 < suffix.len() {
            compress(&self.buffer);
            let mut block = Block::<BlockSize>::default();
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
    pub fn len64_padding_be(&mut self, data_len: u64, compress: impl FnMut(&Block<BlockSize>)) {
        self.digest_pad(0x80, &data_len.to_be_bytes(), compress);
    }

    /// Pad message with 0x80, zeros and 64-bit message length using
    /// little-endian byte order.
    #[inline]
    pub fn len64_padding_le(&mut self, data_len: u64, compress: impl FnMut(&Block<BlockSize>)) {
        self.digest_pad(0x80, &data_len.to_le_bytes(), compress);
    }

    /// Pad message with 0x80, zeros and 128-bit message length using
    /// big-endian byte order.
    #[inline]
    pub fn len128_padding_be(&mut self, data_len: u128, compress: impl FnMut(&Block<BlockSize>)) {
        self.digest_pad(0x80, &data_len.to_be_bytes(), compress);
    }

    /// Pad message with a given padding `P`.
    #[cfg(feature = "block-padding")]
    #[inline]
    pub fn pad_with<P: Padding<BlockSize>>(&mut self) -> Result<&mut Block<BlockSize>, PadError> {
        let pos = self.get_pos();
        P::pad(&mut self.buffer, pos)?;
        self.set_pos_unchecked(0);
        Ok(&mut self.buffer)
    }
}


/// Split message into mutable slice of parallel blocks, blocks, and leftover bytes.
#[inline(always)]
fn to_blocks_mut<N: ArrayLength<u8>>(data: &mut [u8]) -> (&mut [Block<N>], &mut [u8]) {
    let nb = data.len() / N::USIZE;
    let (left, right) = data.split_at_mut(nb * N::USIZE);
    let p = left.as_ptr() as *mut Block<N>;
    // SAFETY: we guarantee that `blocks` does not point outside of `data`
    let blocks = unsafe { slice::from_raw_parts_mut(p, nb) };
    (blocks, right)
}
