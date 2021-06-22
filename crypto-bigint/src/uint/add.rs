//! [`UInt`] addition operations.

use super::UInt;
use crate::{Limb, Wrapping};
use core::ops::{Add, AddAssign};
use subtle::CtOption;

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Computes `a + b + carry`, returning the result along with the new carry.
    #[inline(always)]
    pub const fn adc(&self, rhs: &Self, mut carry: Limb) -> (Self, Limb) {
        let mut limbs = [Limb::ZERO; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let (w, c) = self.limbs[i].adc(rhs.limbs[i], carry);
            limbs[i] = w;
            carry = c;
            i += 1;
        }

        (Self { limbs }, carry)
    }

    /// Perform wrapping addition, discarding overflow.
    pub const fn wrapping_add(&self, rhs: &Self) -> Self {
        self.adc(rhs, Limb::ZERO).0
    }

    /// Perform checked addition, returning a [`CtOption`] which `is_some` only
    /// if the operation did not overflow.
    pub fn checked_add(&self, rhs: &Self) -> CtOption<Self> {
        let (result, carry) = self.adc(rhs, Limb::ZERO);
        CtOption::new(result, carry.is_zero())
    }
}

impl<const LIMBS: usize> Add for Wrapping<UInt<LIMBS>> {
    type Output = Self;

    fn add(self, rhs: Self) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_add(&rhs.0))
    }
}

impl<const LIMBS: usize> Add<&Wrapping<UInt<LIMBS>>> for Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn add(self, rhs: &Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_add(&rhs.0))
    }
}

impl<const LIMBS: usize> Add<Wrapping<UInt<LIMBS>>> for &Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn add(self, rhs: Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_add(&rhs.0))
    }
}

impl<const LIMBS: usize> Add<&Wrapping<UInt<LIMBS>>> for &Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn add(self, rhs: &Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_add(&rhs.0))
    }
}

impl<const LIMBS: usize> AddAssign for Wrapping<UInt<LIMBS>> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<const LIMBS: usize> AddAssign<&Wrapping<UInt<LIMBS>>> for Wrapping<UInt<LIMBS>> {
    fn add_assign(&mut self, other: &Self) {
        *self = *self + other;
    }
}

#[cfg(test)]
mod tests {
    use crate::{Limb, U128};

    #[test]
    fn adc_no_carry() {
        let (res, carry) = U128::ZERO.adc(&U128::ONE, Limb::ZERO);
        assert_eq!(res, U128::ONE);
        assert_eq!(carry, Limb::ZERO);
    }

    #[test]
    fn adc_with_carry() {
        let (res, carry) = U128::MAX.adc(&U128::ONE, Limb::ZERO);
        assert_eq!(res, U128::ZERO);
        assert_eq!(carry, Limb::ONE);
    }

    #[test]
    fn wrapping_add_no_carry() {
        assert_eq!(U128::ZERO.wrapping_add(&U128::ONE), U128::ONE);
    }

    #[test]
    fn wrapping_add_with_carry() {
        assert_eq!(U128::MAX.wrapping_add(&U128::ONE), U128::ZERO);
    }

    #[test]
    fn checked_add_ok() {
        let result = U128::ZERO.checked_add(&U128::ONE);
        assert_eq!(result.unwrap(), U128::ONE);
    }

    #[test]
    fn checked_add_overflow() {
        let result = U128::MAX.checked_add(&U128::ONE);
        assert!(!bool::from(result.is_some()));
    }
}
