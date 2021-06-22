//! Limb addition

use super::{Inner, Limb, Wide};
use crate::{Encoding, Wrapping};
use core::ops::{Add, AddAssign};
use subtle::CtOption;

impl Limb {
    /// Computes `self + rhs + carry`, returning the result along with the new carry.
    #[inline(always)]
    pub const fn adc(self, rhs: Limb, carry: Limb) -> (Limb, Limb) {
        let a = self.0 as Wide;
        let b = rhs.0 as Wide;
        let carry = carry.0 as Wide;
        let ret = a + b + carry;
        (Limb(ret as Inner), Limb((ret >> Self::BIT_SIZE) as Inner))
    }

    /// Perform wrapping addition, discarding overflow.
    #[inline(always)]
    pub const fn wrapping_add(&self, rhs: Self) -> Self {
        Limb(self.0.wrapping_add(rhs.0))
    }

    /// Perform checked addition, returning a [`CtOption`] which `is_some` only
    /// if the operation did not overflow.
    #[inline]
    pub fn checked_add(&self, rhs: Self) -> CtOption<Self> {
        let (result, carry) = self.adc(rhs, Limb::ZERO);
        CtOption::new(result, carry.is_zero())
    }
}

impl Add for Wrapping<Limb> {
    type Output = Self;

    fn add(self, rhs: Self) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_add(rhs.0))
    }
}

impl Add<&Wrapping<Limb>> for Wrapping<Limb> {
    type Output = Wrapping<Limb>;

    fn add(self, rhs: &Wrapping<Limb>) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_add(rhs.0))
    }
}

impl Add<Wrapping<Limb>> for &Wrapping<Limb> {
    type Output = Wrapping<Limb>;

    fn add(self, rhs: Wrapping<Limb>) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_add(rhs.0))
    }
}

impl Add<&Wrapping<Limb>> for &Wrapping<Limb> {
    type Output = Wrapping<Limb>;

    fn add(self, rhs: &Wrapping<Limb>) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_add(rhs.0))
    }
}

impl AddAssign for Wrapping<Limb> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl AddAssign<&Wrapping<Limb>> for Wrapping<Limb> {
    fn add_assign(&mut self, other: &Self) {
        *self = *self + other;
    }
}

#[cfg(test)]
mod tests {
    use crate::Limb;

    #[test]
    fn adc_no_carry() {
        let (res, carry) = Limb::ZERO.adc(Limb::ONE, Limb::ZERO);
        assert_eq!(res, Limb::ONE);
        assert_eq!(carry, Limb::ZERO);
    }

    #[test]
    fn adc_with_carry() {
        let (res, carry) = Limb::MAX.adc(Limb::ONE, Limb::ZERO);
        assert_eq!(res, Limb::ZERO);
        assert_eq!(carry, Limb::ONE);
    }

    #[test]
    fn wrapping_add_no_carry() {
        assert_eq!(Limb::ZERO.wrapping_add(Limb::ONE), Limb::ONE);
    }

    #[test]
    fn wrapping_add_with_carry() {
        assert_eq!(Limb::MAX.wrapping_add(Limb::ONE), Limb::ZERO);
    }

    #[test]
    fn checked_add_ok() {
        let result = Limb::ZERO.checked_add(Limb::ONE);
        assert_eq!(result.unwrap(), Limb::ONE);
    }

    #[test]
    fn checked_add_overflow() {
        let result = Limb::MAX.checked_add(Limb::ONE);
        assert!(!bool::from(result.is_some()));
    }
}
