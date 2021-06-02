//! Macros for defining aliases and relationships between [`UInt`] types.
// TODO(tarcieri): replace these with `const_evaluatable_checked` exprs when stable

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
macro_rules! impl_uint_aliases {
    ($(($name:ident, $bits:expr, $doc:expr)),+) => {
        $(
            #[doc = $doc]
            #[doc="unsigned big integer"]
            pub type $name = UInt<{nlimbs!($bits)}>;

            impl NumBits for $name {
                const NUM_BITS: usize = $bits;
            }

            impl NumBytes for $name {
                const NUM_BYTES: usize = $bits / 8;
            }
        )+
     };
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
macro_rules! impl_concat {
    ($(($name:ident, $bits:expr)),+) => {
        $(
            impl Concat for $name {
                type Output = UInt<{nlimbs!($bits) * 2}>;

                fn concat(&self, rhs: &Self) -> Self::Output {
                    let mut output = Self::Output::default();
                    let (lo, hi) = output.limbs.split_at_mut(self.limbs.len());
                    lo.copy_from_slice(&rhs.limbs);
                    hi.copy_from_slice(&self.limbs);
                    output
                }
            }

            impl From<($name, $name)> for UInt<{nlimbs!($bits) * 2}> {
                fn from(nums: ($name, $name)) -> UInt<{nlimbs!($bits) * 2}> {
                    nums.0.concat(&nums.1)
                }
            }
        )+
     };
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
macro_rules! impl_split {
    ($(($name:ident, $bits:expr)),+) => {
        $(
            impl Split for $name {
                type Output = UInt<{nlimbs!($bits) / 2}>;

                fn split(&self) -> (Self::Output, Self::Output) {
                    let mut hi_out = Self::Output::default();
                    let mut lo_out = Self::Output::default();
                    let (lo_in, hi_in) = self.limbs.split_at(self.limbs.len() / 2);
                    hi_out.limbs.copy_from_slice(&hi_in);
                    lo_out.limbs.copy_from_slice(&lo_in);
                    (hi_out, lo_out)
                }
            }

            impl From<$name> for (UInt<{nlimbs!($bits) / 2}>, UInt<{nlimbs!($bits) / 2}>) {
                fn from(num: $name) -> (UInt<{nlimbs!($bits) / 2}>, UInt<{nlimbs!($bits) / 2}>) {
                    num.split()
                }
            }
        )+
     };
}
