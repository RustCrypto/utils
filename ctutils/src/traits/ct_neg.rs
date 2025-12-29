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

// Impl `CtNeg` for a signed integer (`i*`) type which impls `CtSelect`
macro_rules! impl_signed_ct_neg {
    ( $($int:ty),+ ) => {
        $(
            impl CtNeg for $int {
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

// Impl `CtNeg` for an unsigned integer (`u*`) type which impls `CtSelect`
macro_rules! impl_unsigned_ct_neg {
    ( $($uint:ty),+ ) => {
        $(
            impl CtNeg for $uint {
                #[inline]
                fn ct_neg(&self, choice: Choice) -> Self {
                    let neg = self.wrapping_neg();
                    self.ct_select(&neg, choice)
                }

                #[inline]
                fn ct_neg_assign(&mut self, choice: Choice) {
                     let neg = self.wrapping_neg();
                    self.ct_assign(&neg, choice)
                }
            }
        )+
    };
}

impl_signed_ct_neg!(i8, i16, i32, i64, i128);
impl_unsigned_ct_neg!(u8, u16, u32, u64, u128);

#[cfg(test)]
mod tests {
    /// Test `CtNeg` impl on `i*`
    macro_rules! signed_ct_neg_tests {
         ( $($int:ident),+ ) => {
             $(
                mod $int {
                    use crate::{Choice, CtNeg};

                    #[test]
                    fn ct_neg() {
                        let n: $int = 42;
                        assert_eq!(n, n.ct_neg(Choice::FALSE));
                        assert_eq!(-n, n.ct_neg(Choice::TRUE));
                    }

                    #[test]
                    fn ct_neg_assign() {
                        let n: $int = 42;
                        let mut x = n;
                        x.ct_neg_assign(Choice::FALSE);
                        assert_eq!(n, x);

                        x.ct_neg_assign(Choice::TRUE);
                        assert_eq!(-n, x);
                    }
                }
             )+
        };
    }

    /// Test `CtNeg` impl on `u*`
    macro_rules! unsigned_ct_neg_tests {
         ( $($uint:ident),+ ) => {
             $(
                mod $uint {
                    use crate::{Choice, CtNeg};

                    #[test]
                    fn u32_ct_neg() {
                        let n: $uint = 42;
                        assert_eq!(n, n.ct_neg(Choice::FALSE));
                        assert_eq!(<$uint>::MAX - n + 1, n.ct_neg(Choice::TRUE));
                    }

                    #[test]
                    fn u32_ct_neg_assign() {
                        let n: $uint = 42;
                        let mut x = n;
                        x.ct_neg_assign(Choice::FALSE);
                        assert_eq!(n, x);

                        x.ct_neg_assign(Choice::TRUE);
                        assert_eq!(<$uint>::MAX - n + 1, x);
                    }
                }
             )+
        };
    }

    signed_ct_neg_tests!(i8, i16, i32, i64, i128);
    unsigned_ct_neg_tests!(u8, u16, u32, u64, u128);
}
