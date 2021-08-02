use crate::Block;
use core::slice;
use generic_array::ArrayLength;

/// Split message into slice of blocks and leftover bytes.
#[inline(always)]
pub(crate) fn to_blocks<N: ArrayLength<u8>>(data: &[u8]) -> (&[Block<N>], &[u8]) {
    let nb = data.len() / N::USIZE;
    let (left, right) = data.split_at(nb * N::USIZE);
    let p = left.as_ptr() as *const Block<N>;
    // SAFETY: we guarantee that `blocks` does not point outside of `data`
    let blocks = unsafe { slice::from_raw_parts(p, nb) };
    (blocks, right)
}

/// Split message into mutable slice of parallel blocks, blocks, and leftover bytes.
#[inline(always)]
pub(crate) fn to_blocks_mut<N: ArrayLength<u8>>(data: &mut [u8]) -> (&mut [Block<N>], &mut [u8]) {
    let nb = data.len() / N::USIZE;
    let (left, right) = data.split_at_mut(nb * N::USIZE);
    let p = left.as_ptr() as *mut Block<N>;
    // SAFETY: we guarantee that `blocks` does not point outside of `data`
    let blocks = unsafe { slice::from_raw_parts_mut(p, nb) };
    (blocks, right)
}
