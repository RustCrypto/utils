use hybrid_array::sizes::{U0, U1};

use super::{Array, ArraySize};
use core::{mem::MaybeUninit, ptr, slice};

type Block<N> = MaybeUninit<Array<u8, N>>;

/// Sealed trait for buffer kinds.
pub trait Sealed {
    #[cfg(not(feature = "zeroize"))]
    type Pos: Default + Clone;
    #[cfg(feature = "zeroize")]
    type Pos: Default + Clone + zeroize::Zeroize;

    type Overhead: ArraySize;

    const NAME: &'static str;

    fn get_pos<N: ArraySize>(buf: &Block<N>, pos: &Self::Pos) -> usize;

    fn set_pos<N: ArraySize>(buf: &mut Block<N>, pos: &mut Self::Pos, val: usize);

    /// Invariant guaranteed by a buffer kind, i.e. with correct
    /// buffer code this function always returns true.
    fn invariant(pos: usize, block_size: usize) -> bool;

    /// Split input data into slice of blocks and tail.
    fn split_blocks<N: ArraySize>(data: &[u8]) -> (&[Array<u8, N>], &[u8]);
}

impl Sealed for super::Eager {
    type Pos = ();
    type Overhead = U0;
    const NAME: &'static str = "BlockBuffer<Eager>";

    fn get_pos<N: ArraySize>(buf: &Block<N>, _pos: &Self::Pos) -> usize {
        // SAFETY: last byte in `buf` for eager hashes is always properly initialized
        let pos = unsafe {
            let buf_ptr = buf.as_ptr().cast::<u8>();
            let last_byte_ptr = buf_ptr.add(N::USIZE - 1);
            ptr::read(last_byte_ptr)
        };
        pos as usize
    }

    fn set_pos<N: ArraySize>(buf: &mut Block<N>, _pos: &mut Self::Pos, val: usize) {
        debug_assert!(val <= u8::MAX as usize);
        // SAFETY: we write to the last byte of `buf` which is always safe
        unsafe {
            let buf_ptr = buf.as_mut_ptr().cast::<u8>();
            let last_byte_ptr = buf_ptr.add(N::USIZE - 1);
            ptr::write(last_byte_ptr, val as u8);
        }
    }

    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos < block_size
    }

    #[inline(always)]
    fn split_blocks<N: ArraySize>(data: &[u8]) -> (&[Array<u8, N>], &[u8]) {
        let nb = data.len() / N::USIZE;
        let blocks_len = nb * N::USIZE;

        // SAFETY: we guarantee that created slices do not point outside of `data`
        unsafe {
            let (blocks_raw, tail) = data.split_at_unchecked(blocks_len);
            let blocks = slice::from_raw_parts(blocks_raw.as_ptr().cast(), nb);
            (blocks, tail)
        }
    }
}

impl Sealed for super::Lazy {
    type Pos = u8;
    type Overhead = U1;
    const NAME: &'static str = "BlockBuffer<Lazy>";

    fn get_pos<N: ArraySize>(_buf_val: &Block<N>, pos: &Self::Pos) -> usize {
        *pos as usize
    }

    fn set_pos<N: ArraySize>(_: &mut Block<N>, pos: &mut Self::Pos, val: usize) {
        debug_assert!(val <= u8::MAX as usize);
        *pos = val as u8;
    }

    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos <= block_size
    }

    #[inline(always)]
    fn split_blocks<N: ArraySize>(data: &[u8]) -> (&[Array<u8, N>], &[u8]) {
        if data.is_empty() {
            return (&[], &[]);
        }
        let nb = if data.len() % N::USIZE == 0 {
            data.len() / N::USIZE - 1
        } else {
            data.len() / N::USIZE
        };
        let blocks_len = nb * N::USIZE;

        // SAFETY: we guarantee that created slices do not point outside of `data`
        unsafe {
            let (blocks_raw, tail) = data.split_at_unchecked(blocks_len);
            let blocks = slice::from_raw_parts(blocks_raw.as_ptr().cast(), nb);
            (blocks, tail)
        }
    }
}
