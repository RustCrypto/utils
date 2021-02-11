use crate::{Block, ParBlock};
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
#[allow(clippy::type_complexity)]
#[inline(always)]
pub(crate) fn to_blocks_mut<N: ArrayLength<u8>, M: ArrayLength<Block<N>>>(
    data: &mut [u8],
) -> (&mut [ParBlock<N, M>], &mut [Block<N>], &mut [u8]) {
    let b_size = N::USIZE;
    let pb_size = N::USIZE * M::USIZE;
    let npb = match M::USIZE {
        1 => 0,
        _ => data.len() / pb_size,
    };
    let (pb_slice, data) = data.split_at_mut(npb * pb_size);
    let nb = data.len() / b_size;
    let (b_slice, data) = data.split_at_mut(nb * b_size);
    let pb_ptr = pb_slice.as_mut_ptr() as *mut ParBlock<N, M>;
    let b_ptr = b_slice.as_mut_ptr() as *mut Block<N>;
    // SAFETY: we guarantee that the resulting values do not overlap and do not
    // point outside of the input slice
    unsafe {
        (
            slice::from_raw_parts_mut(pb_ptr, npb),
            slice::from_raw_parts_mut(b_ptr, nb),
            data,
        )
    }
}
