use crate::Choice;
use core::{
    cmp,
    num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128, NonZeroUsize},
};

/// Constant time less than.
pub trait CtLt {
    /// Compute whether `self < other` in constant time.
    #[must_use]
    fn ct_lt(&self, other: &Self) -> Choice;
}

macro_rules! impl_unsigned_ct_lt {
    ( $(($uint:ty, $from_lt:ident)),+ ) => {
        $(
            impl CtLt for $uint {
                #[inline]
                fn ct_lt(&self, other: &Self) -> Choice {
                    // Thumb-1 can branch when converting an overflowing subtraction's borrow flag
                    // into a byte, before `Choice`'s barrier or selection is reached.
                    // Reuse the hardened constructors there.
                    #[cfg(all(target_arch = "arm", not(target_has_atomic = "ptr")))]
                    {
                        Choice::$from_lt(*self, *other)
                    }
                    // Retain native subtraction on other targets.
                    #[cfg(not(all(target_arch = "arm", not(target_has_atomic = "ptr"))))]
                    {
                        let (_, overflow) = self.overflowing_sub(*other);
                        Choice(overflow.into())
                    }
                }
            }
        )+
    };
}

impl_unsigned_ct_lt!((u8, from_u8_lt), (u16, from_u16_lt), (u32, from_u32_lt));

#[cfg(not(target_arch = "riscv32"))]
impl_unsigned_ct_lt!((u64, from_u64_lt));

#[cfg(target_arch = "riscv32")]
impl CtLt for u64 {
    #[inline]
    fn ct_lt(&self, other: &Self) -> Choice {
        // Native wide comparison can short-circuit after caller specialization.
        Choice::from_u64_lt(*self, *other)
    }
}

impl CtLt for usize {
    #[inline]
    #[allow(clippy::cast_possible_truncation, reason = "matches pointer width")]
    fn ct_lt(&self, other: &Self) -> Choice {
        #[cfg(target_pointer_width = "16")]
        let (a, b) = (*self as u16, *other as u16);
        #[cfg(target_pointer_width = "32")]
        let (a, b) = (*self as u32, *other as u32);
        #[cfg(target_pointer_width = "64")]
        let (a, b) = (*self as u64, *other as u64);
        a.ct_lt(&b)
    }
}

impl CtLt for u128 {
    #[inline]
    fn ct_lt(&self, other: &Self) -> Choice {
        Choice::from_u128_lt(*self, *other)
    }
}

/// Impl `CtLt` for `NonZero<T>` by calling `NonZero::get`.
macro_rules! impl_ct_lt_for_nonzero_integer {
    ( $($ty:ty),+ ) => {
        $(
            impl CtLt for $ty {
                #[inline]
                fn ct_lt(&self, other: &Self) -> Choice {
                    self.get().ct_lt(&other.get())
                }
            }
        )+
    };
}

impl_ct_lt_for_nonzero_integer!(
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize
);

impl CtLt for cmp::Ordering {
    #[inline]
    #[allow(clippy::arithmetic_side_effects, clippy::cast_sign_loss)]
    fn ct_lt(&self, other: &Self) -> Choice {
        let a = (*self as i8) + 1;
        let b = (*other as i8) + 1;

        // No impl of `CtLt` for `i8`, so use `u8`
        a.cast_unsigned().ct_lt(&b.cast_unsigned())
    }
}

#[cfg(test)]
mod tests {
    use super::CtLt;
    use core::cmp::Ordering;

    #[test]
    fn ct_lt() {
        let a = 42u64;
        let b = 43u64;
        assert!(!a.ct_lt(&a).to_bool());
        assert!(a.ct_lt(&b).to_bool());
        assert!(!b.ct_lt(&a).to_bool());
    }

    /// Test `CtLt`
    macro_rules! ct_lt_tests {
         ( $($int:ident),+ ) => {
             $(
                mod $int {
                    use super::CtLt;

                    #[test]
                    fn ct_gt() {
                        let a = <$int>::MIN;
                        let b = <$int>::MAX;
                        assert!(!a.ct_lt(&a).to_bool());
                        assert!(a.ct_lt(&b).to_bool());
                        assert!(!b.ct_lt(&a).to_bool());
                    }

                }
             )+
        };
    }

    ct_lt_tests!(u8, u16, u32, u64, u128, usize);

    #[test]
    fn ordering() {
        assert!(!Ordering::Equal.ct_lt(&Ordering::Equal).to_bool());
        assert!(Ordering::Less.ct_lt(&Ordering::Greater).to_bool());
        assert!(!Ordering::Greater.ct_lt(&Ordering::Less).to_bool());
    }
}
