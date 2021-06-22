//! [`UInt`] addition operations.

use super::UInt;
use crate::{Limb, Wrapping};
use core::ops::{Sub, SubAssign};
use subtle::CtOption;

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Computes `a - (b + borrow)`, returning the result along with the new borrow.
    #[inline(always)]
    pub const fn sbb(&self, rhs: &Self, mut borrow: Limb) -> (Self, Limb) {
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let (w, b) = self.limbs[i].sbb(rhs.limbs[i], borrow);
            limbs[i] = w;
            borrow = b;
            i += 1;
        }

        (Self { limbs }, borrow)
    }

    /// Perform wrapping subtraction, discarding underflow and wrapping around
    /// the boundary of the type.
    pub const fn wrapping_sub(&self, rhs: &Self) -> Self {
        self.sbb(rhs, Limb::ZERO).0
    }

    /// Perform checked subtraction, returning a [`CtOption`] which `is_some`
    /// only if the operation did not overflow.
    pub fn checked_sub(&self, rhs: &Self) -> CtOption<Self> {
        let (result, underflow) = self.sbb(rhs, Limb::ZERO);
        CtOption::new(result, underflow.is_zero())
    }
}

impl<const LIMBS: usize> Sub for Wrapping<UInt<LIMBS>> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_sub(&rhs.0))
    }
}

impl<const LIMBS: usize> Sub<&Wrapping<UInt<LIMBS>>> for Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn sub(self, rhs: &Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_sub(&rhs.0))
    }
}

impl<const LIMBS: usize> Sub<Wrapping<UInt<LIMBS>>> for &Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn sub(self, rhs: Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_sub(&rhs.0))
    }
}

impl<const LIMBS: usize> Sub<&Wrapping<UInt<LIMBS>>> for &Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn sub(self, rhs: &Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_sub(&rhs.0))
    }
}

impl<const LIMBS: usize> SubAssign for Wrapping<UInt<LIMBS>> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<const LIMBS: usize> SubAssign<&Wrapping<UInt<LIMBS>>> for Wrapping<UInt<LIMBS>> {
    fn sub_assign(&mut self, other: &Self) {
        *self = *self - other;
    }
}

#[cfg(test)]
mod tests {
    use crate::{Limb, U128};

    #[test]
    fn sbb_no_borrow() {
        let (res, borrow) = U128::ONE.sbb(&U128::ONE, Limb::ZERO);
        assert_eq!(res, U128::ZERO);
        assert_eq!(borrow, Limb::ZERO);
    }

    #[test]
    fn sbb_with_borrow() {
        let (res, borrow) = U128::ZERO.sbb(&U128::ONE, Limb::ZERO);

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
