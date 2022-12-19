use core::slice;
use generic_array::{ArrayLength, GenericArray};

/// Sealed trait for buffer kinds.
pub trait Sealed {
    type Pos: Default + Clone;

    fn get_pos(buf: &[u8], pos: &Self::Pos) -> usize;

    fn set_pos(buf_val: &mut [u8], pos: &mut Self::Pos, val: usize);

    /// Invariant guaranteed by a buffer kind, i.e. with correct
    /// buffer code this function always returns true.
    fn invariant(pos: usize, block_size: usize) -> bool;

    /// Split input data into slice of blocks and tail.
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[GenericArray<u8, N>], &[u8]);
}

impl Sealed for super::Eager {
    type Pos = ();

    fn get_pos(buf: &[u8], _pos: &Self::Pos) -> usize {
        buf[buf.len() - 1] as usize
    }

    fn set_pos(buf: &mut [u8], _pos: &mut Self::Pos, val: usize) {
        debug_assert!(val <= u8::MAX as usize);
        buf[buf.len() - 1] = val as u8;
    }

    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos < block_size
    }

    #[inline(always)]
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[GenericArray<u8, N>], &[u8]) {
        let nb = data.len() / N::USIZE;
        let blocks_len = nb * N::USIZE;
        let tail_len = data.len() - blocks_len;
        // SAFETY: we guarantee that created slices do not point
        // outside of `data`
        unsafe {
            let blocks_ptr = data.as_ptr() as *const GenericArray<u8, N>;
            let tail_ptr = data.as_ptr().add(blocks_len);
            (
                slice::from_raw_parts(blocks_ptr, nb),
                slice::from_raw_parts(tail_ptr, tail_len),
            )
        }
    }
}

impl Sealed for super::Lazy {
    type Pos = u8;

    fn get_pos(_buf_val: &[u8], pos: &Self::Pos) -> usize {
        *pos as usize
    }

    fn set_pos(_buf_val: &mut [u8], pos: &mut Self::Pos, val: usize) {
        debug_assert!(val <= u8::MAX as usize);
        *pos = val as u8;
    }

    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos <= block_size
    }

    #[inline(always)]
    fn split_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[GenericArray<u8, N>], &[u8]) {
        if data.is_empty() {
            return (&[], &[]);
        }
        let (nb, tail_len) = if data.len() % N::USIZE == 0 {
            (data.len() / N::USIZE - 1, N::USIZE)
        } else {
            let nb = data.len() / N::USIZE;
            (nb, data.len() - nb * N::USIZE)
        };
        let blocks_len = nb * N::USIZE;
        // SAFETY: we guarantee that created slices do not point
        // outside of `data`
        unsafe {
            let blocks_ptr = data.as_ptr() as *const GenericArray<u8, N>;
            let tail_ptr = data.as_ptr().add(blocks_len);
            (
                slice::from_raw_parts(blocks_ptr, nb),
                slice::from_raw_parts(tail_ptr, tail_len),
            )
        }
    }
}
