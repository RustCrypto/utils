use super::ArrayLength;

/// Sealed trait for buffer kinds.
pub trait Sealed {
    /// Invariant guaranteed by a buffer kind, i.e. with correct
    /// buffer code this function always returns true.
    fn invariant(pos: usize, block_size: usize) -> bool;

    /// Get the number of blocks for the input data
    fn len<N: ArrayLength<u8>>(data: &[u8]) -> usize;
}

impl Sealed for super::Eager {
    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos < block_size
    }

    #[inline(always)]
    fn len<N: ArrayLength<u8>>(data: &[u8]) -> usize {
        data.len() / N::USIZE
    }
}

impl Sealed for super::Lazy {
    #[inline(always)]
    fn invariant(pos: usize, block_size: usize) -> bool {
        pos <= block_size
    }

    #[inline(always)]
    fn len<N: ArrayLength<u8>>(data: &[u8]) -> usize {
        if data.is_empty() || data.len() % N::USIZE != 0 {
            data.len() / N::USIZE
        } else {
            data.len() / N::USIZE - 1
        }
    }
}
