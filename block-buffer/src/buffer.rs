#[cfg(feature = "block-padding")]
use block_padding::{PadError, Padding};

use crate::{
    utils::{to_blocks, to_blocks_mut},
    Block, DigestBuffer,
};
use core::slice;
use generic_array::ArrayLength;
#[cfg(feature = "inout")]
use inout::InOutBuf;

/// Buffer for block processing of data.
#[derive(Clone, Default)]
pub struct BlockBuffer<BlockSize: ArrayLength<u8>> {
    buffer: Block<BlockSize>,
    pos: usize,
}

impl<BlockSize: ArrayLength<u8>> BlockBuffer<BlockSize> {
    /// XORs `data`. This method is intended for stream cipher implementations.
    #[cfg(feature = "inout")]
    #[inline]
    pub fn xor_data(
        &mut self,
        mut data: InOutBuf<'_, u8>,
        mut process_blocks: impl FnMut(InOutBuf<'_, Block<BlockSize>>),
    ) {
        let pos = self.get_pos();
        let r = self.remaining();
        let n = data.len();
        if pos != 0 {
            if n < r {
                // double slicing allows to remove panic branches
                data.xor(&self.buffer[pos..][..n]);
                self.set_pos_unchecked(pos + n);
                return;
            }
            let (mut left, right) = data.split_at(r);
            data = right;
            left.xor(&self.buffer[pos..]);
        }

        let (blocks, mut leftover) = data.into_chunks();
        process_blocks(blocks);

        let n = leftover.len();
        if n != 0 {
            let mut block = Default::default();
            process_blocks(InOutBuf::from_mut(&mut block));
            leftover.xor(&block[..n]);
            self.buffer = block;
        }
        self.set_pos_unchecked(n);
    }

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

    /// Reset buffer by setting cursor position to zero.
    #[inline]
    pub fn reset(&mut self) {
        self.pos = 0
    }

    /// Return current cursor position.
    #[inline]
    pub fn get_pos(&self) -> usize {
        if self.pos >= BlockSize::USIZE {
            // SAFETY: `pos` is set only to values smaller than block size
            unsafe { core::hint::unreachable_unchecked() }
        }
        self.pos as usize
    }

    /// Set buffer content and cursor position.
    ///
    /// # Panics
    /// If `pos` is bigger or equal to block size.
    pub fn set(&mut self, buf: Block<BlockSize>, pos: usize) {
        assert!(pos < BlockSize::USIZE);
        self.buffer = buf;
        self.pos = pos;
    }

    #[inline]
    fn set_pos_unchecked(&mut self, pos: usize) {
        debug_assert!(pos < BlockSize::USIZE);
        self.pos = pos;
    }
}

impl<B: ArrayLength<u8>> DigestBuffer<B> for BlockBuffer<B> {
    #[inline]
    fn digest_blocks(&mut self, mut input: &[u8], mut compress: impl FnMut(&[Block<B>])) {
        let pos = self.get_pos();
        let r = self.remaining();
        let n = input.len();
        if n < r {
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

        let (blocks, leftover) = to_blocks(input);
        compress(blocks);

        let n = leftover.len();
        self.buffer[..n].copy_from_slice(leftover);
        self.set_pos_unchecked(n);
    }

    #[inline]
    fn reset(&mut self) {
        self.pos = 0;
    }
}
