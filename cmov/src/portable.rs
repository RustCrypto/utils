//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot
//! guarantee that the resulting generated assembly is free of branch
//! instructions.

// TODO(tarcieri): more optimized implementation for small integers

use crate::{Cmov, CmovEq, Condition};

/// Bitwise non-zero: returns `1` if `x != 0`, and otherwise returns `0`.
macro_rules! bitnz {
    ($value:expr, $bits:expr) => {
        core::hint::black_box(($value | $value.wrapping_neg()) >> ($bits - 1))
    };
}

impl Cmov for u16 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u32;
        tmp.cmovnz(&(*value as u32), condition);
        *self = tmp as u16;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u32;
        tmp.cmovz(&(*value as u32), condition);
        *self = tmp as u16;
    }
}

impl CmovEq for u16 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u32).cmovne(&(*rhs as u32), input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u32).cmoveq(&(*rhs as u32), input, output);
    }
}

impl Cmov for u32 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = nzmask32(condition);
        *self = (*self & !mask) | (*value & mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = nzmask32(condition);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u32::BITS) as u8;
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u32::BITS) as u8;
        output.cmovnz(&input, ne ^ 1);
    }
}

impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = nzmask64(condition);
        *self = (*self & !mask) | (*value & mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = nzmask64(condition);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u64::BITS) as u8;
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u64::BITS) as u8;
        output.cmovnz(&input, ne ^ 1);
    }
}

/// Return a [`u32::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
pub fn nzmask32(condition: Condition) -> u32 {
    bitnz!(condition as u32, u32::BITS).wrapping_neg()
}

/// Return a [`u64::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
pub fn nzmask64(condition: Condition) -> u64 {
    bitnz!(condition as u64, u64::BITS).wrapping_neg()
}

#[cfg(test)]
mod tests {
    #[test]
    fn bitnz() {
        assert_eq!(bitnz!(0u8, u8::BITS), 0);
        for i in 1..=u8::MAX {
            assert_eq!(bitnz!(i, u8::BITS), 1);
        }
    }

    #[test]
    fn nzmask32() {
        assert_eq!(super::nzmask32(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::nzmask32(i), u32::MAX);
        }
    }

    #[test]
    fn nzmask64() {
        assert_eq!(super::nzmask64(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::nzmask64(i), u64::MAX);
        }
    }
}
