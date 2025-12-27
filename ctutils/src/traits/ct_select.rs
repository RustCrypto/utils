use crate::Choice;
use cmov::Cmov;
use core::cmp;

/// Constant-time selection: pick between two values based on a given [`Choice`].
pub trait CtSelect: Sized {
    /// Select between `self` and `other` based on `choice`, returning a copy of the value.
    ///
    /// # Returns
    /// - `self` if `choice` is [`Choice::FALSE`].
    /// - `other` if `choice` is [`Choice::TRUE`].
    fn ct_select(&self, other: &Self, choice: Choice) -> Self;

    /// Conditionally assign `other` to `self` if `choice` is [`Choice::TRUE`].
    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        *self = Self::ct_select(self, other, choice);
    }

    /// Conditionally swap `self` and `other` if `choice` is [`Choice::TRUE`].
    fn ct_swap(&mut self, other: &mut Self, choice: Choice) {
        let tmp = self.ct_select(other, choice);
        *other = Self::ct_select(other, self, choice);
        *self = tmp;
    }
}

// Impl `CtSelect` using the `cmov::Cmov` trait
macro_rules! impl_ct_select_with_cmov {
    ( $($ty:ty),+ ) => {
        $(
            impl CtSelect for $ty {
                #[inline]
                fn ct_select(&self, other: &Self, choice: Choice) -> Self {
                    let mut ret = *self;
                    ret.ct_assign(other, choice);
                    ret
                }

                #[inline]
                fn ct_assign(&mut self, other: &Self, choice: Choice) {
                    self.cmovnz(other, choice.into());
                }
            }
        )+
    };
}

impl_ct_select_with_cmov!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl CtSelect for isize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn ct_select(&self, other: &Self, choice: Choice) -> Self {
        (*self as i32).ct_select(&(*other as i32), choice) as isize
    }

    #[cfg(target_pointer_width = "64")]
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
        core::array::from_fn(|i| T::ct_select(&self[i], &other[i], choice))
    }

    fn ct_assign(&mut self, other: &Self, choice: Choice) {
        for (a, b) in self.iter_mut().zip(other) {
            a.ct_assign(b, choice)
        }
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
