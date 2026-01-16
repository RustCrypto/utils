//! Trait impls for core arrays.

use crate::{
    Cmov, CmovEq, Condition,
    utils::{WORD_SIZE, Word, slice_as_chunks},
};

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
impl<const N: usize> Cmov for [u8; N] {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        self.as_mut_slice().cmovnz(value, condition);
    }
}

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`CmovEq`] at the word-level to cut down on the total number of instructions.
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
