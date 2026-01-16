use crate::Choice;
use cmov::CmovEq;
use core::cmp;

#[cfg(feature = "subtle")]
use crate::CtOption;

/// Constant-time equality: like `(Partial)Eq` with [`Choice`] instead of [`bool`].
///
/// Impl'd for: [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`usize`], [`cmp::Ordering`],
/// [`Choice`], and arrays/slices of any type which also impls [`CtEq`].
pub trait CtEq<Rhs = Self>
where
    Rhs: ?Sized,
{
    /// Determine if `self` is equal to `other` in constant-time.
    fn ct_eq(&self, other: &Rhs) -> Choice;

    /// Determine if `self` is NOT equal to `other` in constant-time.
    fn ct_ne(&self, other: &Rhs) -> Choice {
        !self.ct_eq(other)
    }
}

// Impl `CtEq` using the `cmov::CmovEq` trait
macro_rules! impl_ct_eq_with_cmov_eq {
    ( $($ty:ty),+ ) => {
        $(
            impl CtEq for $ty {
                #[inline]
                fn ct_eq(&self, other: &Self) -> Choice {
                    let mut ret = Choice::FALSE;
                    self.cmoveq(other, 1, &mut ret.0);
                    ret
                }
            }
        )+
    };
}

impl_ct_eq_with_cmov_eq!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtEq for isize {
    #[cfg(target_pointer_width = "32")]
    fn ct_eq(&self, other: &Self) -> Choice {
        (*self as i32).ct_eq(&(*other as i32))
    }

    #[cfg(target_pointer_width = "64")]
    fn ct_eq(&self, other: &Self) -> Choice {
        (*self as i64).ct_eq(&(*other as i64))
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtEq for usize {
    #[cfg(target_pointer_width = "32")]
    fn ct_eq(&self, other: &Self) -> Choice {
        (*self as u32).ct_eq(&(*other as u32))
    }

    #[cfg(target_pointer_width = "64")]
    fn ct_eq(&self, other: &Self) -> Choice {
        (*self as u64).ct_eq(&(*other as u64))
    }
}

impl CtEq for cmp::Ordering {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        // `Ordering` is `repr(i8)`, which has a `CtEq` impl
        (*self as i8).ct_eq(&(*other as i8))
    }
}

impl<T: CtEq> CtEq for [T] {
    #[inline]
    fn ct_eq(&self, other: &[T]) -> Choice {
        const {
            assert!(
                size_of::<T>() != 1,
                "use `BytesCtEq::bytes_ct_eq` when working with byte-sized values"
            );
        }

        let mut ret = self.len().ct_eq(&other.len());
        for (a, b) in self.iter().zip(other.iter()) {
            ret &= a.ct_eq(b);
        }
        ret
    }

    #[inline]
    fn ct_ne(&self, other: &[T]) -> Choice {
        const {
            assert!(
                size_of::<T>() != 1,
                "use `BytesCtEq::bytes_ct_ne` when working with byte-sized values"
            );
        }

        !self.ct_eq(other)
    }
}

impl<T: CtEq, const N: usize> CtEq for [T; N] {
    #[inline]
    fn ct_eq(&self, other: &[T; N]) -> Choice {
        self.as_slice().ct_eq(other.as_slice())
    }
}

#[cfg(feature = "subtle")]
impl CtEq for subtle::Choice {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        self.unwrap_u8().ct_eq(&other.unwrap_u8())
    }
}

#[cfg(feature = "subtle")]
impl<T> CtEq for subtle::CtOption<T>
where
    T: CtEq + Default + subtle::ConditionallySelectable,
{
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        CtOption::from(*self).ct_eq(&CtOption::from(*other))
    }
}

#[cfg(test)]
mod tests {
    use super::CtEq;
    use core::cmp::Ordering;

    macro_rules! truth_table {
        ($a:expr, $b:expr, $c:expr) => {
            assert!($a.ct_eq(&$b).to_bool());
            assert!(!$a.ct_eq(&$c).to_bool());
            assert!(!$b.ct_eq(&$c).to_bool());

            assert!(!$a.ct_ne(&$b).to_bool());
            assert!($a.ct_ne(&$c).to_bool());
            assert!($b.ct_ne(&$c).to_bool());
        };
    }

    macro_rules! ct_eq_test {
        ($ty:ty, $name:ident) => {
            #[test]
            fn $name() {
                let a: $ty = 42;
                let b: $ty = 42;
                let c: $ty = 1;
                truth_table!(a, b, c);
            }
        };
    }

    ct_eq_test!(u8, u8_ct_eq);
    ct_eq_test!(u16, u16_ct_eq);
    ct_eq_test!(u32, u32_ct_eq);
    ct_eq_test!(u64, u64_ct_eq);
    ct_eq_test!(u128, u128_ct_eq);
    ct_eq_test!(usize, usize_ct_eq);

    #[test]
    fn array_ct_eq() {
        let a = [1u64, 2, 3];
        let b = [1u64, 2, 3];
        let c = [1u64, 2, 4];
        truth_table!(a, b, c);
    }

    #[test]
    fn ordering_ct_eq() {
        let a = Ordering::Greater;
        let b = Ordering::Greater;
        let c = Ordering::Less;
        truth_table!(a, b, c);
    }

    #[test]
    fn slice_ct_eq() {
        let a: &[u64] = &[1, 2, 3];
        let b: &[u64] = &[1, 2, 3];
        let c: &[u64] = &[1, 2, 4];
        truth_table!(a, b, c);

        // Length mismatches
        assert!(a.ct_ne(&[]).to_bool());
        assert!(a.ct_ne(&[1, 2]).to_bool());
    }
}
