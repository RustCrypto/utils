//! Trait impls for core arrays.

use crate::{Cmov, CmovEq, Condition, slice::cmovnz_slice_unchecked};

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`Cmov`] at the word-level to cut down on the total number of instructions.
impl<const N: usize> Cmov for [u8; N] {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        // "unchecked" means it doesn't check the inputs are equal-length, however they are in this
        // context because they're two equal-sized arrays
        cmovnz_slice_unchecked(self, value, condition);
    }
}

/// Optimized implementation for byte arrays which coalesces them into word-sized chunks first,
/// then performs [`CmovEq`] at the word-level to cut down on the total number of instructions.
impl<const N: usize> CmovEq for [u8; N] {
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        self.as_slice().cmovne(rhs, input, output);
    }
}
