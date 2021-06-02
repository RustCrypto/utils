//! [`UInt`] addition operations.

use super::UInt;
use crate::limb::{self, Limb};
use subtle::{ConstantTimeEq, CtOption};

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Computes `a - (b + borrow)`, returning the result along with the new borrow.
    #[inline(always)]
    pub const fn sbb(&self, rhs: &Self, mut borrow: Limb) -> (Self, Limb) {
        let mut limbs = [0; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let (w, b) = limb::sbb(self.limbs[i], rhs.limbs[i], borrow);
            limbs[i] = w;
            borrow = b;
            i += 1;
        }

        (Self { limbs }, borrow)
    }

    /// Perform wrapping subtraction, discarding underflow and wrapping around
    /// the boundary of the type.
    pub const fn wrapping_sub(&self, rhs: &Self) -> Self {
        self.sbb(rhs, 0).0
    }

    /// Perform checked subtraction, returning [`CtOption`] only if the operation
    /// did not underflow.
    pub fn checked_sub(&self, rhs: &Self) -> CtOption<Self> {
        let (result, underflow) = self.sbb(rhs, 0);
        CtOption::new(result, underflow.ct_eq(&0))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Limb, U128};

    #[test]
    fn sbb_no_borrow() {
        let (res, borrow) = U128::ONE.sbb(&U128::ONE, 0);
        assert_eq!(res, U128::ZERO);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sbb_with_borrow() {
        let (res, borrow) = U128::ZERO.sbb(&U128::ONE, 0);

        assert_eq!(res, U128::MAX);
        assert_eq!(borrow, Limb::MAX);
    }

    #[test]
    fn wrapping_sub_no_borrow() {
        assert_eq!(U128::ONE.wrapping_sub(&U128::ONE), U128::ZERO);
    }

    #[test]
    fn wrapping_sub_with_borrow() {
        assert_eq!(U128::ZERO.wrapping_sub(&U128::ONE), U128::MAX);
    }

    #[test]
    fn checked_sub_ok() {
        let result = U128::ONE.checked_sub(&U128::ONE);
        assert_eq!(result.unwrap(), U128::ZERO);
    }

    #[test]
    fn checked_sub_overflow() {
        let result = U128::ZERO.checked_sub(&U128::ONE);
        assert!(!bool::from(result.is_some()));
    }
}
