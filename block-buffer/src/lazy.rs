use crate::{
    utils::to_blocks,
    Block, DigestBuffer,
};
use generic_array::ArrayLength;

/// Buffer for lazy block processing of data.
#[derive(Clone, Default)]
pub struct LazyBlockBuffer<BlockSize: ArrayLength<u8>> {
    buffer: Block<BlockSize>,
    pos: usize,
}

impl<BlockSize: ArrayLength<u8>> LazyBlockBuffer<BlockSize> {
    /// Pad remaining data with zeros and call `compress` with resulting block.
    pub fn pad_zeros(&mut self) -> &mut Block<BlockSize> {
        let pos = self.get_pos();
        self.buffer[pos..].iter_mut().for_each(|b| *b = 0);
        self.set_pos_unchecked(0);
        &mut self.buffer
    }

    /// Return block if buffer is full, otherwise returns `None`.
    #[inline]
    pub fn get_full_block(&mut self) -> Option<&mut Block<BlockSize>> {
        match self.remaining() {
            0 => Some(&mut self.buffer),
            _ => None,
        }
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
        debug_assert!(self.pos <= BlockSize::USIZE);
        if self.pos > BlockSize::USIZE {
            // SAFETY: `pos` is set only to values smaller or equal to block size
            unsafe { core::hint::unreachable_unchecked() }
        }
        self.pos
    }

    /// Set buffer content and cursor position.
    ///
    /// # Panics
    /// If `pos` is bigger or equal to block size.
    pub fn set(&mut self, buf: Block<BlockSize>, pos: usize) {
        assert!(pos <= BlockSize::USIZE);
        self.buffer = buf;
        self.pos = pos;
    }

    #[inline]
    fn set_pos_unchecked(&mut self, pos: usize) {
        debug_assert!(pos <= BlockSize::USIZE);
        self.pos = pos;
    }
}

impl<B: ArrayLength<u8>> DigestBuffer<B> for LazyBlockBuffer<B> {
    #[inline]
    fn digest_blocks(&mut self, mut input: &[u8], mut compress: impl FnMut(&[Block<B>])) {
        let pos = self.get_pos();
        let r = self.remaining();
        let n = input.len();
        if n <= r {
            // double slicing allows to remove panic branches
            self.buffer[pos..][..n].copy_from_slice(input);
            self.set_pos_unchecked(pos + n);
            return;
        }
        if pos != 0 {
            let (left, right) = input.split_at(r);
            input = right;
            self.buffer[pos..].copy_from_slice(left);
            compress(core::slice::from_ref(&self.buffer));
        }

        let (blocks, leftover) = to_blocks_lazy(input);
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

fn to_blocks_lazy<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]) {
    let (mut blocks, mut leftover) = to_blocks(data);
    if leftover.is_empty() {
        debug_assert!(!blocks.is_empty());
        let m = blocks.len() - 1;
        // SAFETY: at this stage `input` always contains at least one byte,
        // so either `leftover` is not empty or we have at least one block
        unsafe {
            leftover = blocks.get_unchecked(m);
            blocks = blocks.get_unchecked(..m);
        }
    }
    (blocks, leftover)
}
