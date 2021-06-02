//! `subtle`-based constant time operations on [`UInt`].

use super::UInt;
use crate::Limb;
use subtle::{
    Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
};

impl<const LIMBS: usize> ConditionallySelectable for UInt<LIMBS> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut limbs = [0; LIMBS];

        for i in 0..LIMBS {
            limbs[i] = Limb::conditional_select(&a.limbs[0], &b.limbs[0], choice);
        }

        Self { limbs }
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
        let underflow = other.sbb(self, 0).1;
        !underflow.ct_eq(&0)
    }
}

impl<const LIMBS: usize> ConstantTimeLess for UInt<LIMBS> {
    fn ct_lt(&self, other: &Self) -> Choice {
        let underflow = self.sbb(other, 0).1;
        !underflow.ct_eq(&0)
    }
}

#[cfg(test)]
mod tests {
    use crate::U128;
    use subtle::{
        Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
    };

    #[test]
    fn conditional_select() {
        let a = U128::ZERO;
        let b = U128::MAX;

        assert_eq!(U128::conditional_select(&a, &b, Choice::from(0)), a);
        assert_eq!(U128::conditional_select(&a, &b, Choice::from(1)), b);
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
