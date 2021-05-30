//! Checked arithmetic.

use crate::UInt;
use core::ops::{Add, AddAssign, Deref, Mul, MulAssign, Sub, SubAssign};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// Provides intentionally-checked arithmetic on `T`.
///
/// Internally this leverages the [`CtOption`] type from the [`subtle`] crate
/// in order to handle overflows in constant time.
#[derive(Copy, Clone, Debug)]
pub struct Checked<T>(pub CtOption<T>);

impl<T> Checked<T> {
    /// Create a new checked arithmetic wrapper for the given value.
    pub fn new(val: T) -> Self {
        Self(CtOption::new(val, Choice::from(1)))
    }
}

impl<T> Default for Checked<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Deref for Checked<T> {
    type Target = CtOption<T>;

    fn deref(&self) -> &CtOption<T> {
        &self.0
    }
}

impl<T: ConditionallySelectable> ConditionallySelectable for Checked<T> {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(CtOption::conditional_select(&a.0, &b.0, choice))
    }
}

impl<T: ConstantTimeEq> ConstantTimeEq for Checked<T> {
    #[inline]
    fn ct_eq(&self, rhs: &Self) -> Choice {
        self.0.ct_eq(&rhs.0)
    }
}

impl<const LIMBS: usize> Add for Checked<UInt<LIMBS>> {
    type Output = Self;

    fn add(self, rhs: Self) -> Checked<UInt<LIMBS>> {
        Checked(self.0.and_then(|a| rhs.0.and_then(|b| a.checked_add(&b))))
    }
}

impl<const LIMBS: usize> AddAssign for Checked<UInt<LIMBS>> {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<const LIMBS: usize> Sub for Checked<UInt<LIMBS>> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Checked<UInt<LIMBS>> {
        Checked(self.0.and_then(|a| rhs.0.and_then(|b| a.checked_sub(&b))))
    }
}

impl<const LIMBS: usize> SubAssign for Checked<UInt<LIMBS>> {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<const LIMBS: usize> Mul for Checked<UInt<LIMBS>> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Checked<UInt<LIMBS>> {
        Checked(self.0.and_then(|a| rhs.0.and_then(|b| a.checked_mul(&b))))
    }
}

impl<const LIMBS: usize> MulAssign for Checked<UInt<LIMBS>> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}
