use super::{ArrayLength, Block};
use core::slice;

/// Sealed trait for buffer kinds.
pub trait Sealed {
    fn assert_pos(pos: usize, block_size: usize);
    fn debug_assert_pos(pos: usize, block_size: usize);
    fn input_check(input_len: usize, rem: usize) -> bool;
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]);
    unsafe fn unreachable_hint(pos: usize, block_size: usize);
}

impl Sealed for super::Eager {
    #[inline(always)]
    fn assert_pos(pos: usize, block_size: usize) {
        assert!(pos < block_size)
    }

    #[inline(always)]
    fn debug_assert_pos(pos: usize, block_size: usize) {
        debug_assert!(pos < block_size)
    }

    #[inline(always)]
    fn input_check(input_len: usize, rem: usize) -> bool {
        input_len < rem
    }

    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]) {
        let nb = data.len() / N::USIZE;
        let (left, right) = data.split_at(nb * N::USIZE);
        let p = left.as_ptr() as *const Block<N>;
        // SAFETY: we guarantee that `blocks` does not point outside of `data`
        let blocks = unsafe { slice::from_raw_parts(p, nb) };
        (blocks, right)
    }

    #[inline(always)]
    unsafe fn unreachable_hint(pos: usize, block_size: usize) {
        if pos >= block_size {
            core::hint::unreachable_unchecked()
        }
    }
}

impl Sealed for super::Lazy {
    #[inline(always)]
    fn assert_pos(pos: usize, block_size: usize) {
        assert!(pos <= block_size)
    }

    #[inline(always)]
    fn debug_assert_pos(pos: usize, block_size: usize) {
        debug_assert!(pos <= block_size)
    }

    #[inline(always)]
    fn input_check(input_len: usize, rem: usize) -> bool {
        input_len <= rem
    }

    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]) {
        let mut nb = data.len() / N::USIZE;
        if data.len() % N::USIZE == 0 && nb != 0 {
            nb -= 1;
        }
        let (left, right) = data.split_at(nb * N::USIZE);
        let p = left.as_ptr() as *const Block<N>;
        // SAFETY: we guarantee that `blocks` does not point outside of `data`
        let blocks = unsafe { slice::from_raw_parts(p, nb) };
        (blocks, right)
    }

    #[inline(always)]
    unsafe fn unreachable_hint(pos: usize, block_size: usize) {
        if block_size != 0 && pos > block_size {
            core::hint::unreachable_unchecked()
        }
    }
}
