//! [`UInt`] comparisons.
//!
//! By default these are all constant-time and use the `subtle` crate.

use super::UInt;
use crate::Limb;
use core::cmp::Ordering;
use subtle::{Choice, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess};

impl<const LIMBS: usize> UInt<LIMBS> {
    /// Determine if this [`UInt`] is equal to zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }

    /// Is this [`UInt`] an odd number?
    #[inline]
    pub fn is_odd(&self) -> Choice {
        self.limbs
            .first()
            .map(|limb| limb.is_odd())
            .unwrap_or_else(|| Choice::from(0))
    }
}

impl<const LIMBS: usize> ConstantTimeEq for UInt<LIMBS> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.limbs
            .iter()
            .zip(other.limbs.iter())
            .fold(Choice::from(1), |acc, (a, b)| acc & a.ct_eq(b))
    }
}

impl<const LIMBS: usize> ConstantTimeGreater for UInt<LIMBS> {
    fn ct_gt(&self, other: &Self) -> Choice {
        let underflow = other.sbb(self, Limb::ZERO).1;
        !underflow.is_zero()
    }
}

impl<const LIMBS: usize> ConstantTimeLess for UInt<LIMBS> {
    fn ct_lt(&self, other: &Self) -> Choice {
        let underflow = self.sbb(other, Limb::ZERO).1;
        !underflow.is_zero()
    }
}

impl<const LIMBS: usize> Eq for UInt<LIMBS> {}

impl<const LIMBS: usize> Ord for UInt<LIMBS> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut n = 0i8;
        n -= self.ct_lt(other).unwrap_u8() as i8;
        n += self.ct_gt(other).unwrap_u8() as i8;

        match n {
            -1 => Ordering::Less,
            1 => Ordering::Greater,
            _ => {
                debug_assert_eq!(n, 0);
                debug_assert!(bool::from(self.ct_eq(other)));
                Ordering::Equal
            }
        }
    }
}

impl<const LIMBS: usize> PartialOrd for UInt<LIMBS> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const LIMBS: usize> PartialEq for UInt<LIMBS> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::U128;
    use subtle::{ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess};

    #[test]
    fn is_zero() {
        assert!(bool::from(U128::ZERO.is_zero()));
        assert!(!bool::from(U128::ONE.is_zero()));
        assert!(!bool::from(U128::MAX.is_zero()));
    }

    #[test]
    fn is_odd() {
        assert!(!bool::from(U128::ZERO.is_odd()));
        assert!(bool::from(U128::ONE.is_odd()));
        assert!(bool::from(U128::MAX.is_odd()));
    }

    #[test]
    fn ct_eq() {
        let a = U128::ZERO;
        let b = U128::MAX;

        assert!(bool::from(a.ct_eq(&a)));
        assert!(!bool::from(a.ct_eq(&b)));
        assert!(!bool::from(b.ct_eq(&a)));
        assert!(bool::from(b.ct_eq(&b)));
    }

    #[test]
    fn ct_gt() {
        let a = U128::ZERO;
        let b = U128::ONE;
        let c = U128::MAX;

        assert!(bool::from(b.ct_gt(&a)));
        assert!(bool::from(c.ct_gt(&a)));
        assert!(bool::from(c.ct_gt(&b)));

        assert!(!bool::from(a.ct_gt(&a)));
        assert!(!bool::from(b.ct_gt(&b)));
        assert!(!bool::from(c.ct_gt(&c)));

        assert!(!bool::from(a.ct_gt(&b)));
        assert!(!bool::from(a.ct_gt(&c)));
        assert!(!bool::from(b.ct_gt(&c)));
    }

    #[test]
    fn ct_lt() {
        let a = U128::ZERO;
        let b = U128::ONE;
        let c = U128::MAX;

        assert!(bool::from(a.ct_lt(&b)));
        assert!(bool::from(a.ct_lt(&c)));
        assert!(bool::from(b.ct_lt(&c)));

        assert!(!bool::from(a.ct_lt(&a)));
        assert!(!bool::from(b.ct_lt(&b)));
        assert!(!bool::from(c.ct_lt(&c)));

        assert!(!bool::from(b.ct_lt(&a)));
        assert!(!bool::from(c.ct_lt(&a)));
        assert!(!bool::from(c.ct_lt(&b)));
    }
}
