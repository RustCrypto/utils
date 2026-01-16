#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::checked_conversions,
    clippy::implicit_saturating_sub,
    clippy::integer_division_remainder_used,
    clippy::mod_module_files,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::unwrap_used,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unused_lifetimes,
    unused_qualifications
)]

#[macro_use]
mod macros;

#[cfg(not(miri))]
#[cfg(target_arch = "aarch64")]
mod aarch64;
mod array;
#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")),
    miri
))]
mod portable;
mod slice;
#[cfg(not(miri))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Condition
pub type Condition = u8;

/// Conditional move
pub trait Cmov {
    /// Move if non-zero.
    ///
    /// Moves `value` to `self` in constant-time if `condition` is non-zero.
    fn cmovnz(&mut self, value: &Self, condition: Condition);

    /// Move if zero.
    ///
    /// Moves `value` to `self` in constant-time if `condition` is equal to zero.
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let nz = masknz!(condition: Condition);
        self.cmovnz(value, !nz)
    }
}

/// Conditional move with equality comparison
pub trait CmovEq {
    /// Move if both inputs are not equal.
    ///
    /// Moves `input` to `output` in constant-time if `self` and `rhs` are NOT equal.
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let mut tmp = 1u8;
        self.cmoveq(rhs, 0u8, &mut tmp);
        tmp.cmoveq(&1u8, input, output);
    }

    /// Move if both inputs are equal.
    ///
    /// Moves `input` to `output` in constant-time if `self` and `rhs` are equal.
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let mut tmp = 1u8;
        self.cmovne(rhs, 0u8, &mut tmp);
        tmp.cmoveq(&1, input, output);
    }
}

impl Cmov for u8 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = u16::from(*self);
        tmp.cmovnz(&u16::from(*value), condition);
        debug_assert!(u8::try_from(tmp).is_ok());
        *self = (tmp & 0xFF) as u8;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = u16::from(*self);
        tmp.cmovz(&u16::from(*value), condition);
        debug_assert!(u8::try_from(tmp).is_ok());
        *self = (tmp & 0xFF) as u8;
    }
}

impl Cmov for u128 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovnz(&((*value & 0xFFFF_FFFF_FFFF_FFFF) as u64), condition);
        hi.cmovnz(&((*value >> 64) as u64), condition);

        *self = u128::from(lo) | (u128::from(hi) << 64);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovz(&((*value & 0xFFFF_FFFF_FFFF_FFFF) as u64), condition);
        hi.cmovz(&((*value >> 64) as u64), condition);

        *self = u128::from(lo) | (u128::from(hi) << 64);
    }
}

impl CmovEq for u8 {
    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        u16::from(*self).cmoveq(&u16::from(*rhs), input, output);
    }

    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        u16::from(*self).cmovne(&u16::from(*rhs), input, output);
    }
}

impl CmovEq for u128 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let hi = (*self >> 64) as u64;

        let mut tmp = 1u8;
        lo.cmovne(&((*rhs & 0xFFFF_FFFF_FFFF_FFFF) as u64), 0, &mut tmp);
        hi.cmovne(&((*rhs >> 64) as u64), 0, &mut tmp);
        tmp.cmoveq(&0, input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let lo = (*self & 0xFFFF_FFFF_FFFF_FFFF) as u64;
        let hi = (*self >> 64) as u64;

        let mut tmp = 1u8;
        lo.cmovne(&((*rhs & 0xFFFF_FFFF_FFFF_FFFF) as u64), 0, &mut tmp);
        hi.cmovne(&((*rhs >> 64) as u64), 0, &mut tmp);
        tmp.cmoveq(&1, input, output);
    }
}

// Impl `Cmov*` by first casting to unsigned then using the unsigned `Cmov` impls
// TODO(tarcieri): use `cast_unsigned`/`cast_signed` to get rid of the `=> u*`
macro_rules! impl_cmov_traits_for_signed_ints {
    ( $($int:ty => $uint:ty),+ ) => {
        $(
            impl Cmov for $int {
                #[inline]
                fn cmovnz(&mut self, value: &Self, condition: Condition) {
                    let mut tmp = *self as $uint;
                    tmp.cmovnz(&(*value as $uint), condition);
                    *self = tmp as $int;
                }

                #[inline]
                fn cmovz(&mut self, value: &Self, condition: Condition) {
                    let mut tmp = *self as $uint;
                    tmp.cmovz(&(*value as $uint), condition);
                    *self = tmp as $int;
                }
            }

            impl CmovEq for $int {
                #[inline]
                fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                    (*self as $uint).cmoveq(&(*rhs as $uint), input, output);
                }

                #[inline]
                fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
                    (*self as $uint).cmovne(&(*rhs as $uint), input, output);
                }
            }
        )+
    };
}

impl_cmov_traits_for_signed_ints!(i8 => u8, i16 => u16, i32 => u32, i64 => u64, i128 => u128);
