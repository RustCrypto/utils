//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot
//! guarantee that the resulting generated assembly is free of branch
//! instructions.

// TODO(tarcieri): more optimized implementation for small integers

use crate::{Cmov, CmovEq, Condition};
use core::hint::black_box;

/// Bitwise non-zero: returns `1` if `x != 0`, and otherwise returns `0`.
macro_rules! bitnz {
    ($value:expr, $bits:expr) => {
        ($value | $value.wrapping_neg()) >> ($bits - 1)
    };
}

impl Cmov for u16 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovnz(&(*value as u64), condition);
        *self = tmp as u16;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovz(&(*value as u64), condition);
        *self = tmp as u16;
    }
}

impl CmovEq for u16 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmovne(&(*rhs as u64), input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmoveq(&(*rhs as u64), input, output);
    }
}

impl Cmov for u32 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovnz(&(*value as u64), condition);
        *self = tmp as u32;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovz(&(*value as u64), condition);
        *self = tmp as u32;
    }
}

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmovne(&(*rhs as u64), input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmoveq(&(*rhs as u64), input, output);
    }
}

impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = black_box((bitnz!(condition, u8::BITS) as u64).wrapping_sub(1));
        *self = (*self & mask) | (*value & !mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = black_box((1 ^ bitnz!(condition, u8::BITS) as u64).wrapping_sub(1));
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u64::BITS) as u8;
        output.cmovnz(&input, black_box(ne));
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u64::BITS) as u8;
        output.cmovnz(&input, black_box(ne ^ 1));
    }
}
