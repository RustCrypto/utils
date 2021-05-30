//! Wrapping arithmetic.

use crate::UInt;
use core::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

/// Provides intentionally-wrapped arithmetic on `T`.
///
/// This is analogous to [`core::num::Wrapping`] but allows this crate to
/// define trait impls for this type.
#[derive(Copy, Clone, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Wrapping<T>(pub T);

impl<T: fmt::Display> fmt::Display for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Binary> fmt::Binary for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::Octal> fmt::Octal for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::LowerHex> fmt::LowerHex for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: fmt::UpperHex> fmt::UpperHex for Wrapping<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ConditionallySelectable> ConditionallySelectable for Wrapping<T> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Wrapping(T::conditional_select(&a.0, &b.0, choice))
    }
}

impl<T: ConstantTimeEq> ConstantTimeEq for Wrapping<T> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
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

impl<const LIMBS: usize> Mul for Wrapping<UInt<LIMBS>> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_mul(&rhs.0))
    }
}

impl<const LIMBS: usize> Mul<&Wrapping<UInt<LIMBS>>> for Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn mul(self, rhs: &Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_mul(&rhs.0))
    }
}

impl<const LIMBS: usize> Mul<Wrapping<UInt<LIMBS>>> for &Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn mul(self, rhs: Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_mul(&rhs.0))
    }
}

impl<const LIMBS: usize> Mul<&Wrapping<UInt<LIMBS>>> for &Wrapping<UInt<LIMBS>> {
    type Output = Wrapping<UInt<LIMBS>>;

    fn mul(self, rhs: &Wrapping<UInt<LIMBS>>) -> Wrapping<UInt<LIMBS>> {
        Wrapping(self.0.wrapping_mul(&rhs.0))
    }
}

impl<const LIMBS: usize> MulAssign for Wrapping<UInt<LIMBS>> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<const LIMBS: usize> MulAssign<&Wrapping<UInt<LIMBS>>> for Wrapping<UInt<LIMBS>> {
    fn mul_assign(&mut self, other: &Self) {
        *self = *self * other;
    }
}
