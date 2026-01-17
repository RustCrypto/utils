//! Trait impls for core arrays.

use crate::{Cmov, CmovEq, Condition};

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
        self.as_slice().cmovne(rhs, input, output);
    }
}
