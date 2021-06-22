//! Limb subtraction

use super::{Inner, Limb, Wide};
use crate::{Encoding, Wrapping};
use core::ops::{Sub, SubAssign};
use subtle::CtOption;

impl Limb {
    /// Computes `self - (rhs + borrow)`, returning the result along with the new borrow.
    #[inline(always)]
    pub const fn sbb(self, rhs: Limb, borrow: Limb) -> (Limb, Limb) {
        let a = self.0 as Wide;
        let b = rhs.0 as Wide;
        let borrow = (borrow.0 >> (Self::BIT_SIZE - 1)) as Wide;
        let ret = a.wrapping_sub(b + borrow);
        (Limb(ret as Inner), Limb((ret >> Self::BIT_SIZE) as Inner))
    }

    /// Perform wrapping subtraction, discarding underflow and wrapping around
    /// the boundary of the type.
    #[inline(always)]
    pub const fn wrapping_sub(&self, rhs: Self) -> Self {
        Limb(self.0.wrapping_sub(rhs.0))
    }

    /// Perform checked subtraction, returning a [`CtOption`] which `is_some`
    /// only if the operation did not overflow.
    #[inline]
    pub fn checked_sub(&self, rhs: Self) -> CtOption<Self> {
        let (result, underflow) = self.sbb(rhs, Limb::ZERO);
        CtOption::new(result, underflow.is_zero())
    }
}

impl Sub for Wrapping<Limb> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_sub(rhs.0))
    }
}

impl Sub<&Wrapping<Limb>> for Wrapping<Limb> {
    type Output = Wrapping<Limb>;

    fn sub(self, rhs: &Wrapping<Limb>) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_sub(rhs.0))
    }
}

impl Sub<Wrapping<Limb>> for &Wrapping<Limb> {
    type Output = Wrapping<Limb>;

    fn sub(self, rhs: Wrapping<Limb>) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_sub(rhs.0))
    }
}

impl Sub<&Wrapping<Limb>> for &Wrapping<Limb> {
    type Output = Wrapping<Limb>;

    fn sub(self, rhs: &Wrapping<Limb>) -> Wrapping<Limb> {
        Wrapping(self.0.wrapping_sub(rhs.0))
    }
}

impl SubAssign for Wrapping<Limb> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl SubAssign<&Wrapping<Limb>> for Wrapping<Limb> {
    fn sub_assign(&mut self, other: &Self) {
        *self = *self - other;
    }
}

#[cfg(test)]
mod tests {
    use crate::Limb;

    #[test]
    fn sbb_no_borrow() {
        let (res, borrow) = Limb::ONE.sbb(Limb::ONE, Limb::ZERO);
        assert_eq!(res, Limb::ZERO);
        assert_eq!(borrow, Limb::ZERO);
    }

    #[test]
    fn sbb_with_borrow() {
        let (res, borrow) = Limb::ZERO.sbb(Limb::ONE, Limb::ZERO);

        assert_eq!(res, Limb::MAX);
        assert_eq!(borrow, Limb::MAX);
    }

    #[test]
    fn wrapping_sub_no_borrow() {
        assert_eq!(Limb::ONE.wrapping_sub(Limb::ONE), Limb::ZERO);
    }

    #[test]
    fn wrapping_sub_with_borrow() {
        assert_eq!(Limb::ZERO.wrapping_sub(Limb::ONE), Limb::MAX);
    }

    #[test]
    fn checked_sub_ok() {
        let result = Limb::ONE.checked_sub(Limb::ONE);
        assert_eq!(result.unwrap(), Limb::ZERO);
    }

    #[test]
    fn checked_sub_overflow() {
        let result = Limb::ZERO.checked_sub(Limb::ONE);
        assert!(!bool::from(result.is_some()));
    }
}
