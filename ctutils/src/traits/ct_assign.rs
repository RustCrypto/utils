use crate::{Choice, CtSelect};
use cmov::Cmov;
use core::cmp;

/// Constant-time conditional assignment: assign a given value to another based on a [`Choice`].
pub trait CtAssign {
    /// Conditionally assign `other` to `self` if `choice` is [`Choice::TRUE`].
    fn ct_assign(&mut self, other: &Self, choice: Choice);
}

// Impl `CtAssign` using the `cmov::Cmov` trait
macro_rules! impl_ct_assign_with_cmov {
    ( $($ty:ty),+ ) => {
        $(
            impl CtAssign for $ty {
                #[inline]
                fn ct_assign(&mut self, other: &Self, choice: Choice) {
                    self.cmovnz(other, choice.into());
                }
            }
        )+
    };
}

impl_ct_assign_with_cmov!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtAssign for isize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtAssign for usize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }
}

impl CtAssign for cmp::Ordering {
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }
}

impl<T> CtAssign for [T]
where
    T: CtAssign,
{
    #[inline]
    #[track_caller]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        assert_eq!(
            self.len(),
            other.len(),
            "source slice length ({}) does not match destination slice length ({})",
            other.len(),
            self.len()
        );

        for (a, b) in self.iter_mut().zip(other) {
            a.ct_assign(b, choice)
        }
    }
}

impl<T, const N: usize> CtAssign for [T; N]
where
    T: CtAssign,
{
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        self.as_mut_slice().ct_assign(other, choice);
    }
}

#[cfg(feature = "subtle")]
impl CtAssign for subtle::Choice {
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }
}

#[cfg(feature = "subtle")]
impl<T> CtAssign for subtle::CtOption<T>
where
    T: Default + subtle::ConditionallySelectable,
{
    #[inline]
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        use subtle::ConditionallySelectable as _;
        self.conditional_assign(other, choice.into());
    }
}
