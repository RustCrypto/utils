//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot guarantee that the
//! resulting generated assembly is free of branch instructions.
//!
//! For select platforms we use `asm!` for mask generation which should largely mitigate the
//! optimizer potentially inserting branches.

use crate::{Cmov, CmovEq, Condition};

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
        let mask = masknz32(condition);
        *self = (*self & !mask) | (*value & mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = masknz32(condition);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = testnz32(self ^ rhs) as u8;
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = testnz32(self ^ rhs) as u8;
        output.cmovnz(&input, ne ^ 1);
    }
}

impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = masknz64(condition);
        *self = (*self & !mask) | (*value & mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = masknz64(condition);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = testnz64(self ^ rhs) as u8;
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = testnz64(self ^ rhs) as u8;
        output.cmovnz(&input, ne ^ 1);
    }
}

/// Writes a function which performs a conditional test on its input, returning `0` if the input
/// is zero, or `1` if the input is non-zero.
///
/// Includes a `black_box` on the output to coerce the optimizer into not speculating on the
/// return value.
macro_rules! testnz {
    ($name:ident, $int:ident) => {
        /// Returns `0` if `x` is `0`, otherwise returns `1`.
        pub fn $name(mut x: $int) -> $int {
            x |= x.wrapping_neg();
            core::hint::black_box(x >> ($int::BITS - 1))
        }
    };
}

testnz!(testnz32, u32);
testnz!(testnz64, u64);

/// Return a [`u32::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
pub fn masknz32(condition: Condition) -> u32 {
    testnz32(condition as u32).wrapping_neg()
}

/// Return a [`u64::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
pub fn masknz64(condition: Condition) -> u64 {
    testnz64(condition as u64).wrapping_neg()
}

#[cfg(test)]
mod tests {
    #[test]
    fn testnz32() {
        assert_eq!(super::testnz32(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::testnz32(i as u32), 1);
        }
    }

    #[test]
    fn testnz64() {
        assert_eq!(super::testnz64(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::testnz64(i as u64), 1);
        }
    }

    #[test]
    fn masknz32() {
        assert_eq!(super::masknz32(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::masknz32(i), u32::MAX);
        }
    }

    #[test]
    fn masknz64() {
        assert_eq!(super::masknz64(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::masknz64(i), u64::MAX);
        }
    }
}
