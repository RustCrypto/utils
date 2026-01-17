use crate::Choice;
use core::{
    cmp,
    num::{NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128},
};

/// Constant time greater than.
pub trait CtGt {
    /// Compute whether `self > other` in constant time.
    #[must_use]
    fn ct_gt(&self, other: &Self) -> Choice;
}

// Impl `CtGt` using overflowing subtraction
macro_rules! impl_unsigned_ct_gt {
    ( $($uint:ty),+ ) => {
        $(
            impl CtGt for $uint {
                #[inline]
                fn ct_gt(&self, other: &Self) -> Choice {
                    let (_, overflow) = other.overflowing_sub(*self);
                    Choice(overflow.into())
                }
            }
        )+
    };
}

impl_unsigned_ct_gt!(u8, u16, u32, u64, u128);

/// Impl `CtGt` for `NonZero<T>` by calling `NonZero::get`.
macro_rules! impl_ct_gt_for_nonzero_integer {
    ( $($ty:ty),+ ) => {
        $(
            impl CtGt for $ty {
                #[inline]
                fn ct_gt(&self, other: &Self) -> Choice {
                    self.get().ct_gt(&other.get())
                }
            }
        )+
    };
}

impl_ct_gt_for_nonzero_integer!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128);

impl CtGt for cmp::Ordering {
    #[inline]
    fn ct_gt(&self, other: &Self) -> Choice {
        // No impl of `CtGt` for `i8`, so use `u8`
        let a = (*self as i8) + 1;
        let b = (*other as i8) + 1;
        (a as u8).ct_gt(&(b as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::CtGt;
    use core::cmp::Ordering;

    #[test]
    fn ct_gt() {
        let a = 42u64;
        let b = 43u64;
        assert!(!a.ct_gt(&a).to_bool());
        assert!(!a.ct_gt(&b).to_bool());
        assert!(b.ct_gt(&a).to_bool());
    }

    #[test]
    fn ordering() {
        assert!(!Ordering::Equal.ct_gt(&Ordering::Equal).to_bool());
        assert!(!Ordering::Less.ct_gt(&Ordering::Greater).to_bool());
        assert!(Ordering::Greater.ct_gt(&Ordering::Less).to_bool());
    }
}
