//! Utility functions.
//!
//! Note that support of big-endian targets is somewhat sub-optimal since we prioritized
//! simplicity of the implementation.

#[inline(always)]
pub(crate) fn absorb_full<const N: usize, const RATE: usize>(
    state: &mut [u64; N],
    block: &[u8; RATE],
) {
    const {
        assert!(size_of::<[u8; RATE]>() <= size_of::<[u64; N]>());
        assert!(RATE % size_of::<u64>() == 0);
    };

    let chunks = block.chunks_exact(size_of::<u64>());
    assert!(chunks.remainder().is_empty());

    for (dst, chunk) in state.iter_mut().zip(chunks) {
        let chunk = chunk.try_into().expect("chunk has correct length");
        *dst ^= u64::from_le_bytes(chunk);
    }
}

#[inline(always)]
pub(crate) fn absorb_partial<const N: usize, const RATE: usize>(
    state: &mut [u64; N],
    offset: usize,
    data: &[u8],
) {
    const {
        assert!(size_of::<[u8; RATE]>() <= size_of::<[u64; N]>());
        assert!(RATE % size_of::<u64>() == 0);
    };

    if cfg!(target_endian = "little") {
        // SAFETY: casting of `&mut [u64; N]` into `&mut [u8; M]` is safe if
        // `size_of::<[u8; M]>() <= size_of::<[u64; N]>())`
        let sub_state: &mut [u8; RATE] = unsafe { &mut *(state.as_mut_ptr().cast()) };

        let dst = &mut sub_state[offset..][..data.len()];

        for i in 0..dst.len() {
            dst[i] ^= data[i];
        }
    } else {
        let mut buf = [0u8; RATE];
        buf[offset..][..data.len()].copy_from_slice(data);

        let chunks = buf.chunks_exact(size_of::<u64>());
        assert!(chunks.remainder().is_empty());

        for (dst, chunk) in state.iter_mut().zip(chunks) {
            let chunk = chunk.try_into().expect("chunk has correct length");
            *dst ^= u64::from_le_bytes(chunk);
        }
    }
}

#[inline(always)]
pub(crate) fn squeeze_read_full<const N: usize, const RATE: usize>(
    state: &[u64; N],
    dst: &mut [u8; RATE],
) {
    const {
        assert!(size_of::<[u8; RATE]>() <= size_of::<[u64; N]>());
        assert!(RATE % size_of::<u64>() == 0);
    };

    let mut dst_chunks = dst.chunks_exact_mut(size_of::<u64>());
    for (src, dst_chunk) in state.iter().zip(&mut dst_chunks) {
        dst_chunk.copy_from_slice(&src.to_le_bytes());
    }
    assert!(dst_chunks.into_remainder().is_empty());
}

#[inline(always)]
pub(crate) fn squeeze_read_partial<const N: usize, const RATE: usize>(
    state: &[u64; N],
    offset: usize,
    dst: &mut [u8],
) {
    const {
        assert!(size_of::<[u8; RATE]>() <= size_of::<[u64; N]>());
        assert!(RATE % size_of::<u64>() == 0);
    };

    let mut buf = [0u8; RATE];

    let sub_state: &[u8; RATE] = if cfg!(target_endian = "little") {
        // SAFETY: casting of `&[u64; N]` into `&[u8; M]` is safe if
        // `size_of::<[u8; M]>() <= size_of::<[u64; N]>())`
        unsafe { &*(state.as_ptr().cast::<[u8; RATE]>()) }
    } else {
        let mut chunks = buf.chunks_exact_mut(size_of::<u64>());
        for (src, dst) in state.iter().zip(&mut chunks) {
            dst.copy_from_slice(&src.to_le_bytes());
        }
        assert!(chunks.into_remainder().is_empty());
        &buf
    };

    let src = &sub_state[offset..][..dst.len()];
    dst.copy_from_slice(src);
}

#[inline(always)]
pub(crate) fn squeeze_xor_full<const N: usize, const RATE: usize>(
    state: &[u64; N],
    dst: &mut [u8; RATE],
) {
    const {
        assert!(size_of::<[u8; RATE]>() <= size_of::<[u64; N]>());
        assert!(RATE % size_of::<u64>() == 0);
    };

    let mut dst_chunks = dst.chunks_exact_mut(size_of::<u64>());
    for (src, dst_chunk) in state.iter().zip(&mut dst_chunks) {
        let dst_chunk: &mut [u8; 8] = dst_chunk.try_into().expect("chunk has correct size");
        let chunk_val = u64::from_le_bytes(*dst_chunk);
        *dst_chunk = (chunk_val ^ src).to_le_bytes();
    }
    assert!(dst_chunks.into_remainder().is_empty());
}

#[inline(always)]
pub(crate) fn squeeze_xor_partial<const N: usize, const RATE: usize>(
    state: &[u64; N],
    offset: usize,
    dst: &mut [u8],
) {
    const {
        assert!(size_of::<[u8; RATE]>() <= size_of::<[u64; N]>());
        assert!(RATE % size_of::<u64>() == 0);
    };

    let mut buf = [0u8; RATE];

    let sub_state: &[u8; RATE] = if cfg!(target_endian = "little") {
        // SAFETY: casting of `&[u64; N]` into `&[u8; M]` is safe if
        // `size_of::<[u8; M]>() <= size_of::<[u64; N]>())`
        unsafe { &*(state.as_ptr().cast::<[u8; RATE]>()) }
    } else {
        let mut chunks = buf.chunks_exact_mut(size_of::<u64>());
        for (src, dst) in state.iter().zip(&mut chunks) {
            dst.copy_from_slice(&src.to_le_bytes());
        }
        assert!(chunks.into_remainder().is_empty());
        &buf
    };

    let src = &sub_state[offset..][..dst.len()];
    for i in 0..dst.len() {
        dst[i] ^= src[i];
    }
}
