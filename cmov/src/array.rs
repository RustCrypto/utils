use crate::{
    Cmov, CmovEq, Condition,
    utils::{slice_as_chunks, slice_as_chunks_mut},
};

// Uses 64-bit words on 64-bit targets, 32-bit everywhere else
#[cfg(not(target_pointer_width = "64"))]
type Word = u32;
#[cfg(target_pointer_width = "64")]
type Word = u64;
const WORD_SIZE: usize = size_of::<Word>();

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
///
/// With compile-time knowledge of `N`, the compiler should also be able to unroll the loops in
/// cases where efficiency would benefit, reducing the implementation to a sequence of word-sized
/// [`Cmov`] ops (and if `N` isn't word-aligned, followed by a series of 1-byte ops).
impl<const N: usize> Cmov for [u8; N] {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let (self_chunks, self_remainder) = slice_as_chunks_mut::<u8, WORD_SIZE>(self);
        let (value_chunks, value_remainder) = slice_as_chunks::<u8, WORD_SIZE>(value);

        for (self_chunk, value_chunk) in self_chunks.iter_mut().zip(value_chunks.iter()) {
            let mut a = Word::from_ne_bytes(*self_chunk);
            let b = Word::from_ne_bytes(*value_chunk);
            a.cmovnz(&b, condition);
            self_chunk.copy_from_slice(&a.to_ne_bytes());
        }

        // Process the remainder a byte-at-a-time.
        for (a, b) in self_remainder.iter_mut().zip(value_remainder.iter()) {
            a.cmovnz(b, condition);
        }
    }
}

impl<const N: usize> CmovEq for [u8; N] {
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let (self_chunks, self_remainder) = slice_as_chunks::<u8, WORD_SIZE>(self);
        let (rhs_chunks, rhs_remainder) = slice_as_chunks::<u8, WORD_SIZE>(rhs);

        for (self_chunk, rhs_chunk) in self_chunks.iter().zip(rhs_chunks.iter()) {
            let a = Word::from_ne_bytes(*self_chunk);
            let b = Word::from_ne_bytes(*rhs_chunk);
            a.cmovne(&b, input, output);
        }

        // Process the remainder a byte-at-a-time.
        for (a, b) in self_remainder.iter().zip(rhs_remainder.iter()) {
            a.cmovne(b, input, output);
        }
    }
}
