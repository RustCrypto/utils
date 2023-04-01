#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(not(miri))]
#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(any(
    not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")),
    miri
))]
mod portable;
#[cfg(not(miri))]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Condition
pub type Condition = u8;

/// Conditional move
pub trait Cmov {
    /// Move if non-zero.
    ///
    /// Uses a `test` instruction to check if the given `condition` value is
    /// equal to zero, conditionally moves `value` to `self` when `condition` is
    /// equal to zero.
    fn cmovnz(&mut self, value: &Self, condition: Condition);

    /// Move if zero.
    ///
    /// Uses a `cmp` instruction to check if the given `condition` value is
    /// equal to zero, and if so, conditionally moves `value` to `self`
    /// when `condition` is equal to zero.
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        self.cmovnz(value, !condition)
    }
}

/// Conditional move with equality comparison
pub trait CmovEq {
    /// Move if both inputs are equal.
    ///
    /// Uses a `xor` instruction to compare the two values, and
    /// conditionally moves `input` to `output` when they are equal.
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition);

    /// Move if both inputs are not equal.
    ///
    /// Uses a `xor` instruction to compare the two values, and
    /// conditionally moves `input` to `output` when they are not equal.
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let mut tmp = 1u8;
        self.cmoveq(rhs, 0u8, &mut tmp);
        tmp.cmoveq(&1u8, input, output);
    }
}

impl Cmov for u8 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u16;
        tmp.cmovnz(&(*value as u16), condition);
        *self = tmp as u8;
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u16;
        tmp.cmovz(&(*value as u16), condition);
        *self = tmp as u8;
    }
}

impl CmovEq for u8 {
    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u16).cmoveq(&(*rhs as u16), input, output);
    }

    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u16).cmovne(&(*rhs as u16), input, output);
    }
}

impl Cmov for u128 {
    #[inline(always)]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut lo = (*self & u64::MAX as u128) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovnz(&((*value & u64::MAX as u128) as u64), condition);
        hi.cmovnz(&((*value >> 64) as u64), condition);

        *self = (lo as u128) | (hi as u128) << 64;
    }

    #[inline(always)]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut lo = (*self & u64::MAX as u128) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovz(&((*value & u64::MAX as u128) as u64), condition);
        hi.cmovz(&((*value >> 64) as u64), condition);

        *self = (lo as u128) | (hi as u128) << 64;
    }
}

impl CmovEq for u128 {
    #[inline(always)]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let lo = (*self & u64::MAX as u128) as u64;
        let hi = (*self >> 64) as u64;

        let mut tmp = 1u8;
        lo.cmovne(&((*rhs & u64::MAX as u128) as u64), 0, &mut tmp);
        hi.cmovne(&((*rhs >> 64) as u64), 0, &mut tmp);
        tmp.cmoveq(&0, input, output);
    }

    #[inline(always)]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let lo = (*self & u64::MAX as u128) as u64;
        let hi = (*self >> 64) as u64;

        let mut tmp = 1u8;
        lo.cmovne(&((*rhs & u64::MAX as u128) as u64), 0, &mut tmp);
        hi.cmovne(&((*rhs >> 64) as u64), 0, &mut tmp);
        tmp.cmoveq(&1, input, output);
    }
}
