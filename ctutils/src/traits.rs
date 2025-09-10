use crate::Choice;
use cmov::CmovEq;
use core::cmp;

/// `Eq` with `Choice` instead of bool`.
pub trait CtEq {
    /// Equality
    fn ct_eq(&self, other: &Self) -> Choice;

    /// Inequality
    fn ct_ne(&self, other: &Self) -> Choice {
        !self.ct_eq(other)
    }
}

macro_rules! impl_cteq_with_cmov {
    ( $($uint:ty),+ ) => {
        $(
            impl CtEq for $uint {
                #[inline]
                fn ct_eq(&self, other: &Self) -> Choice {
                    let mut ret = 0;
                    self.cmoveq(other, 1, &mut ret);
                    ret.into()
                }
            }
        )+
    };
}

impl_cteq_with_cmov!(u8, u16, u32, u64, u128);

impl CtEq for bool {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        u8::from(*self).ct_eq(&u8::from(*other))
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

impl CtEq for Choice {
    #[inline]
    fn ct_eq(&self, rhs: &Choice) -> Choice {
        !(*self ^ *rhs)
    }
}

impl CtEq for cmp::Ordering {
    #[inline]
    fn ct_eq(&self, other: &Self) -> Choice {
        // `Ordering` is `repr(i8)`, however we don't impl `CtEq` for `i8` (though we could easily).
        // Instead, this casts `Ordering` to a `u8`, which preserves bit pattern equality. Then we
        // are able to use the `ct_eq` impl on `u8`, which is backed by `cmov`.
        (*self as u8).ct_eq(&(*other as u8))
    }
}

impl<T: CtEq> CtEq for [T] {
    /// Compare slices in constant time.
    ///
    /// NOTE: exits early in the event of a length mismatch.
    #[inline]
    fn ct_eq(&self, rhs: &[T]) -> Choice {
        if self.len() != rhs.len() {
            return Choice::FALSE;
        }

        let mut ret = Choice::TRUE;
        for (a, b) in self.iter().zip(rhs.iter()) {
            ret &= a.ct_eq(b);
        }
        ret
    }
}
