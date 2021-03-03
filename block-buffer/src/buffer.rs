#[cfg(feature = "block-padding")]
use block_padding::Padding;

use crate::{
    utils::{to_blocks, to_blocks_mut},
    Block, DigestBuffer, InvalidLength, ParBlock,
};
use core::slice;
use generic_array::{typenum::U1, ArrayLength};

/// Buffer for block processing of data.
#[derive(Clone, Default)]
pub struct BlockBuffer<BlockSize: ArrayLength<u8>> {
    buffer: Block<BlockSize>,
    pos: usize,
}

impl<BlockSize: ArrayLength<u8>> BlockBuffer<BlockSize> {
    /// Core method for `xor_data` and `set_data` methods.
    ///
    /// If `N` is equal to 1, the `gen_blocks` function is not used.
    fn process_data<S, N: ArrayLength<Block<BlockSize>>>(
        &mut self,
        mut data: &mut [u8],
        state: &mut S,
        mut process: impl FnMut(&mut [u8], &[u8]),
        mut gen_block: impl FnMut(&mut S) -> Block<BlockSize>,
        mut gen_blocks: impl FnMut(&mut S) -> ParBlock<BlockSize, N>,
    ) {
        let pos = self.get_pos();
        let r = self.remaining();
        let n = data.len();
        if pos != 0 {
            if n < r {
                // double slicing allows to remove panic branches
                process(data, &self.buffer[pos..][..n]);
                self.set_pos_unchecked(pos + n);
                return;
            }
            let (left, right) = data.split_at_mut(r);
            data = right;
            process(left, &self.buffer[pos..]);
        }

        let (par_blocks, blocks, leftover) = to_blocks_mut::<BlockSize, N>(data);
        for pb in par_blocks {
            let blocks = gen_blocks(state);
            for i in 0..N::USIZE {
                process(&mut pb[i], &blocks[i]);
            }
        }

        for block in blocks {
            process(block, &gen_block(state));
        }

        let n = leftover.len();
        if n != 0 {
            let block = gen_block(state);
            process(leftover, &block[..n]);
            self.buffer = block;
        }
        self.set_pos_unchecked(n);
    }

    /// XORs `data` using the provided state and block generation functions.
    ///
    /// This method is intended for stream cipher implementations. If `N` is
    /// equal to 1, the `gen_blocks` function is not used.
    #[inline]
    pub fn par_xor_data<S, N: ArrayLength<Block<BlockSize>>>(
        &mut self,
        data: &mut [u8],
        state: &mut S,
        gen_block: impl FnMut(&mut S) -> Block<BlockSize>,
        gen_blocks: impl FnMut(&mut S) -> ParBlock<BlockSize, N>,
    ) {
        self.process_data(data, state, xor, gen_block, gen_blocks);
    }

    /// Simplified version of the [`par_xor_data`][BlockBuffer::par_xor_data] method, with `N = 1`.
    #[inline]
    pub fn xor_data(&mut self, data: &mut [u8], mut gen_block: impl FnMut() -> Block<BlockSize>) {
        // note: the unrachable panic should be removed by compiler since
        // with `N = 1` the second closure is not used
        self.process_data(data, &mut gen_block, xor, |f| f(), unreachable);
    }

    /// Set `data` to generated blocks.
    #[inline]
    pub fn set_data(&mut self, data: &mut [u8], mut gen_block: impl FnMut() -> Block<BlockSize>) {
        // note: the unrachable panic should be removed by compiler since
        // with `N = 1` the second closure is not used
        self.process_data(data, &mut gen_block, set, |f| f(), unreachable);
    }

    /// Process `data` in blocks and write result to `out_buf`, storing
    /// leftovers for future use.
    #[inline]
    pub fn block_mode_processing<'a>(
        &mut self,
        mut data: &[u8],
        buf: &'a mut [u8],
        mut process: impl FnMut(&mut [Block<BlockSize>]),
    ) -> Result<&'a [u8], InvalidLength> {
        let pos = self.get_pos();
        let rem = self.remaining();
        let mut blocks_processed = 0;
        let (_, mut buf_blocks, _) = to_blocks_mut::<BlockSize, U1>(buf);
        if pos != 0 {
            let n = data.len();
            if n < rem {
                // double slicing allows to remove panic branches
                self.buffer[pos..][..n].copy_from_slice(data);
                self.set_pos_unchecked(pos + n);
                return Ok(&buf[..0]);
            }
            if buf_blocks.is_empty() {
                return Err(InvalidLength);
            }

            let (l, r) = buf_blocks.split_at_mut(1);
            let buf_block = &mut l[0];
            buf_blocks = r;
            let (l, r) = data.split_at(rem);
            data = r;

            buf_block[..pos].copy_from_slice(&self.buffer[..pos]);
            buf_block[pos..].copy_from_slice(l);

            process(slice::from_mut(buf_block));
            blocks_processed += 1;
        }

        let (data_blocks, leftover) = to_blocks(data);
        let buf_blocks = buf_blocks
            .get_mut(..data_blocks.len())
            .ok_or(InvalidLength)?;
        buf_blocks.clone_from_slice(data_blocks);
        process(buf_blocks);
        blocks_processed += buf_blocks.len();

        let n = leftover.len();
        self.buffer[..n].copy_from_slice(leftover);
        self.set_pos_unchecked(n);

        let res = unsafe {
            let res_len = BlockSize::USIZE * blocks_processed;
            // SAFETY: number of processed blocks never exceeds capacity of `buf`
            debug_assert!(buf.len() >= res_len);
            buf.get_unchecked(..res_len)
        };
        Ok(res)
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
            let mut block: Block<BlockSize> = Default::default();
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
    pub fn pad_with<P: Padding<BlockSize>>(&mut self) -> &mut Block<BlockSize> {
        let pos = self.get_pos();
        P::pad(&mut self.buffer, pos);
        self.set_pos_unchecked(0);
        &mut self.buffer
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
        debug_assert!(self.pos < BlockSize::USIZE);
        if self.pos >= BlockSize::USIZE {
            // SAFETY: `pos` is set only to values smaller than block size
            unsafe { core::hint::unreachable_unchecked() }
        }
        self.pos
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

#[inline(always)]
fn xor(a: &mut [u8], b: &[u8]) {
    debug_assert_eq!(a.len(), b.len());
    a.iter_mut().zip(b.iter()).for_each(|(a, &b)| *a ^= b);
}

#[inline(always)]
fn set(a: &mut [u8], b: &[u8]) {
    a.copy_from_slice(b);
}

fn unreachable<S, B: ArrayLength<u8>>(_: &mut S) -> ParBlock<B, U1> {
    unreachable!();
}
