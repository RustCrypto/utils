use crate::{Choice, CtSelect};
use cmov::Cmov;
use core::{
    cmp,
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU128,
    },
};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

/// Constant-time conditional assignment: assign a given value to another based on a [`Choice`].
pub trait CtAssign<Rhs: ?Sized = Self> {
    /// Conditionally assign `rhs` to `self` if `choice` is [`Choice::TRUE`].
    fn ct_assign(&mut self, rhs: &Rhs, choice: Choice);
}

/// Impl `CtAssign` using the `CtSelect` trait.
///
/// In cases where `CtSelect` is more straightforward to implement, but you want to use a provided
/// implementation of `CtAssign` based on it, you can use this macro to write it for you.
#[macro_export]
macro_rules! impl_ct_assign_with_ct_select {
    ( $($ty:ty),+ ) => {
        $(
            impl CtAssign for $ty {
                #[inline]
                fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
                    *self = Self::ct_select(self, rhs, choice);
                }
            }
        )+
    };
}

impl_ct_assign_with_ct_select!(
    cmp::Ordering,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128
);

/// Impl `CtAssign` using the `cmov::Cmov` trait
macro_rules! impl_ct_assign_with_cmov {
    ( $($ty:ty),+ ) => {
        $(
            impl CtAssign for $ty {
                #[inline]
                fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
                    self.cmovnz(rhs, choice.into());
                }
            }
        )+
    };
}

impl_ct_assign_with_cmov!(
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    u128,
    [i8],
    [i16],
    [i32],
    [i64],
    [i128],
    [u8],
    [u16],
    [u32],
    [u64],
    [u128]
);

#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
impl_ct_assign_with_cmov!(isize, usize);

impl<T, const N: usize> CtAssign for [T; N]
where
    [T]: CtAssign,
{
    #[inline]
    fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
        self.as_mut_slice().ct_assign(rhs, choice);
    }
}

#[cfg(feature = "alloc")]
impl<T> CtAssign for Box<T>
where
    T: CtAssign,
{
    #[inline]
    #[track_caller]
    fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
        (**self).ct_assign(rhs, choice);
    }
}

#[cfg(feature = "alloc")]
impl<T> CtAssign for Box<[T]>
where
    [T]: CtAssign,
{
    #[inline]
    #[track_caller]
    fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
        self.ct_assign(&**rhs, choice);
    }
}

#[cfg(feature = "alloc")]
impl<T> CtAssign<[T]> for Box<[T]>
where
    [T]: CtAssign,
{
    #[inline]
    #[track_caller]
    fn ct_assign(&mut self, rhs: &[T], choice: Choice) {
        (**self).ct_assign(rhs, choice);
    }
}

#[cfg(feature = "alloc")]
impl<T> CtAssign for Vec<T>
where
    [T]: CtAssign,
{
    #[inline]
    #[track_caller]
    fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
        self.ct_assign(rhs.as_slice(), choice);
    }
}

#[cfg(feature = "alloc")]
impl<T> CtAssign<[T]> for Vec<T>
where
    [T]: CtAssign,
{
    #[inline]
    #[track_caller]
    fn ct_assign(&mut self, rhs: &[T], choice: Choice) {
        self.as_mut_slice().ct_assign(rhs, choice);
    }
}

#[cfg(feature = "subtle")]
impl CtAssign for subtle::Choice {
    #[inline]
    fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
        *self = Self::ct_select(self, rhs, choice);
    }
}

#[cfg(feature = "subtle")]
impl<T> CtAssign for subtle::CtOption<T>
where
    T: Default + subtle::ConditionallySelectable,
{
    #[inline]
    fn ct_assign(&mut self, rhs: &Self, choice: Choice) {
        use subtle::ConditionallySelectable as _;
        self.conditional_assign(rhs, choice.into());
    }
}
