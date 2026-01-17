use crate::{Choice, CtAssign};
use core::{
    cmp,
    num::{
        NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128, NonZeroU8, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU128,
    },
};

#[cfg(feature = "subtle")]
use crate::CtOption;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

/// Constant-time selection: pick between two values based on a given [`Choice`].
pub trait CtSelect: CtAssign + Sized {
    /// Select between `self` and `other` based on `choice`, returning a copy of the value.
    ///
    /// # Returns
    /// - `self` if `choice` is [`Choice::FALSE`].
    /// - `other` if `choice` is [`Choice::TRUE`].
    fn ct_select(&self, other: &Self, choice: Choice) -> Self;

    /// Conditionally swap `self` and `other` if `choice` is [`Choice::TRUE`].
    fn ct_swap(&mut self, other: &mut Self, choice: Choice) {
        let tmp = self.ct_select(other, choice);
        *other = Self::ct_select(other, self, choice);
        *self = tmp;
    }
}

/// Impl `CtSelect` using the `CtAssign` trait.
///
/// In cases where `CtAssign` is more straightforward to implement, but you want to use a provided
/// implementation of `CtSelect` based on it, you can use this macro to write it for you.
///
/// Requires the provided type(s) impl `Clone`.
#[macro_export]
macro_rules! impl_ct_select_with_ct_assign {
    ( $($ty:ty),+ ) => {
        $(
            impl CtSelect for $ty {
                #[inline]
                fn ct_select(&self, other: &Self, choice: Choice) -> Self {
                    let mut ret = self.clone();
                    ret.ct_assign(other, choice);
                    ret
                }
            }
        )+
    };
}

impl_ct_select_with_ct_assign!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

/// Impl `CtSelect` for `NonZero<T>` by calling the `CtSelect` impl for `T`.
macro_rules! impl_ct_select_for_nonzero_integer {
    ( $($nzint:ident),+ ) => {
        $(
             impl CtSelect for $nzint {
                #[inline]
                fn ct_select(&self, rhs: &Self, choice: Choice) -> Self {
                    let n = self.get().ct_select(&rhs.get(), choice);

                    // SAFETY: we are constructing `NonZero` from a value we obtained from
                    // `NonZero::get`, which ensures it's non-zero.
                    #[allow(unsafe_code)]
                    unsafe { $nzint::new_unchecked(n) }
                }
            }
        )+
    };
}

impl_ct_select_for_nonzero_integer!(
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

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtSelect for isize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        (*self as i32).ct_select(&(*other as i32), choice) as isize
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        (*self as i64).ct_select(&(*other as i64), choice) as isize
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtSelect for usize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        (*self as u32).ct_select(&(*other as u32), choice) as usize
    }

    #[cfg(target_pointer_width = "64")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        (*self as u64).ct_select(&(*other as u64), choice) as usize
    }
}

impl CtSelect for cmp::Ordering {
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        // `Ordering` is `#[repr(i8)]` where:
        //
        // - `Less` => -1
        // - `Equal` => 0
        // - `Greater` => 1
        //
        // Given this, it's possible to operate on orderings as if they're `i8`, which allows us to
        // use the `CtSelect` impl on `i8` to select between them.
        let ret = (*self as i8).ct_select(&(*other as i8), choice);

        // SAFETY: `Ordering` is `#[repr(i8)]` and `ret` has been assigned to
        // a value which was originally a valid `Ordering` then cast to `i8`
        #[allow(trivial_casts, unsafe_code)]
        unsafe {
            *(&ret as *const i8).cast::<Self>()
        }
    }
}

impl<T, const N: usize> CtSelect for [T; N]
where
    T: CtSelect,
{
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        const {
            assert!(
                size_of::<T>() != 1,
                "use `BytesCtSelect::bytes_ct_select` when working with byte-sized values"
            );
        }

        core::array::from_fn(|i| T::ct_select(&self[i], &other[i], choice))
    }
}

#[cfg(feature = "alloc")]
impl<T> CtSelect for Box<T>
where
    T: CtSelect,
{
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        Box::new(T::ct_select(&**self, &**other, choice))
    }
}

#[cfg(feature = "alloc")]
impl<T> CtSelect for Box<[T]>
where
    T: Clone + CtSelect,
{
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        let mut ret = self.clone();
        ret.ct_assign(other, choice);
        ret
    }
}

#[cfg(feature = "alloc")]
impl<T> CtSelect for Vec<T>
where
    T: Clone + CtSelect,
{
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        let mut ret = self.clone();
        ret.ct_assign(other, choice);
        ret
    }
}

#[cfg(feature = "subtle")]
impl CtSelect for subtle::Choice {
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        Choice::from(*self)
            .ct_select(&Choice::from(*other), choice)
            .into()
    }
}

#[cfg(feature = "subtle")]
impl<T> CtSelect for subtle::CtOption<T>
where
    T: CtSelect + Default + subtle::ConditionallySelectable,
{
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        CtOption::from(*self)
            .ct_select(&CtOption::from(*other), choice)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Choice, CtSelect, cmp};

    macro_rules! ct_select_test {
        ($ty:ty, $name:ident) => {
            #[test]
            fn $name() {
                let a: $ty = 1;
                let b: $ty = 2;
                assert_eq!(a.ct_select(&b, Choice::FALSE), a);
                assert_eq!(a.ct_select(&b, Choice::TRUE), b);
            }
        };
    }

    ct_select_test!(u8, u8_ct_select);
    ct_select_test!(u16, u16_ct_select);
    ct_select_test!(u32, u32_ct_select);
    ct_select_test!(u64, u64_ct_select);
    ct_select_test!(u128, u128_ct_select);

    #[test]
    fn ordering_ct_select() {
        let a = cmp::Ordering::Less;
        let b = cmp::Ordering::Greater;
        assert_eq!(a.ct_select(&b, Choice::FALSE), a);
        assert_eq!(a.ct_select(&b, Choice::TRUE), b);
    }
}
