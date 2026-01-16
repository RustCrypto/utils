//! Trait impls for core slices.

use crate::utils::{WORD_SIZE, Word, slice_as_chunks, slice_as_chunks_mut};
use crate::{Cmov, CmovEq, Condition};

/// Optimized implementation for byte slices which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
impl Cmov for [u8] {
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

impl<T: CmovEq> CmovEq for [T] {
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        // Short-circuit the comparison if the slices are of different lengths, and set the output
        // condition to the input condition.
        if self.len() != rhs.len() {
            *output = input;
            return;
        }

        // Compare each byte.
        for (a, b) in self.iter().zip(rhs.iter()) {
            a.cmovne(b, input, output);
        }
    }
}
