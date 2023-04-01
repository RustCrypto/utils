//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot
//! guarantee that the resulting generated assembly is free of branch
//! instructions.

// TODO(tarcieri): more optimized implementation for small integers

use crate::{Cmov, CmovEq, Condition};

impl Cmov for u16 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovnz(&(*value as u64), condition);
        *self = tmp as u16;
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovz(&(*value as u64), condition);
        *self = tmp as u16;
    }
}

impl CmovEq for u16 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmovne(&(*rhs as u64), input, output);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmoveq(&(*rhs as u64), input, output);
    }
}

impl Cmov for u32 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovnz(&(*value as u64), condition);
        *self = tmp as u32;
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovz(&(*value as u64), condition);
        *self = tmp as u32;
    }
}

impl CmovEq for u32 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmovne(&(*rhs as u64), input, output);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmoveq(&(*rhs as u64), input, output);
    }
}

impl Cmov for u64 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = is_non_zero(condition).wrapping_sub(1);
        *self = (*self & mask) | (*value & !mask);
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = (1 ^ is_non_zero(condition)).wrapping_sub(1);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u64 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        output.cmovnz(&input, (self ^ rhs) as u8);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        output.cmovz(&input, (self ^ rhs) as u8);
    }
}

/// Check if the given condition value is non-zero
///
/// # Returns
/// - `condition` is zero: `0`
/// - `condition` is non-zero: `1`
#[inline(always)]
fn is_non_zero(condition: Condition) -> u64 {
    const SHIFT_BITS: usize = core::mem::size_of::<u64>() - 1;
    let condition = condition as u64;
    ((condition | (!condition).wrapping_add(1)) >> SHIFT_BITS) & 1
}
