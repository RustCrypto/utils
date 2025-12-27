use crate::{Choice, CtSelect};

/// Constant-time conditional negation: negates a value when `choice` is [`Choice::TRUE`].
pub trait CtNeg: Sized {
    /// Conditionally negate `self`, returning `-self` if `choice` is [`Choice::TRUE`], or `self`
    /// otherwise.
    fn ct_neg(&self, choice: Choice) -> Self;

    /// Conditionally negate `self` in-place, replacing it with `-self` if `choice` is
    /// [`Choice::TRUE`].
    fn ct_neg_assign(&mut self, choice: Choice) {
        *self = self.ct_neg(choice);
    }
}

// Impl `CtNeg` for an integer type which impls `CtSelect`
macro_rules! impl_ct_neg {
    ( $($ty:ty),+ ) => {
        $(
            impl CtNeg for $ty {
                #[inline]
                fn ct_neg(&self, choice: Choice) -> Self {
                    let neg = -*self;
                    self.ct_select(&neg, choice)
                }

                #[inline]
                fn ct_neg_assign(&mut self, choice: Choice) {
                    let neg = -*self;
                    self.ct_assign(&neg, choice)
                }
            }
        )+
    };
}

impl_ct_neg!(i8, i16, i32, i64, i128);

// TODO(tarcieri): test all signed integer types
#[cfg(test)]
mod tests {
    use super::{Choice, CtNeg};

    #[test]
    fn i64_ct_neg() {
        assert_eq!(42, 42.ct_neg(Choice::FALSE));
        assert_eq!(-42, 42.ct_neg(Choice::TRUE));
    }

    #[test]
    fn i64_ct_neg_assign() {
        let mut n = 42;
        n.ct_neg_assign(Choice::FALSE);
        assert_eq!(42, n);

        n.ct_neg_assign(Choice::TRUE);
        assert_eq!(-42, n);
    }
}
