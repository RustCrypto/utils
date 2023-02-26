#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(not(any(target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
mod portable;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86;

/// Condition
pub type Condition = u8;

/// Conditional move
// TODO(tarcieri): make one of `cmovz`/`cmovnz` a provided method which calls the other?
pub trait Cmov {
    /// Move if zero.
    ///
    /// Uses a `cmp` instruction to check if the given `condition` value is
    /// equal to zero, and if so, conditionally moves `value` to `self`
    /// when `condition` is equal to zero.
    fn cmovz(&mut self, value: Self, condition: Condition);

    /// Move if non-zero.
    ///
    /// Uses a `test` instruction to check if the given `condition` value is
    /// equal to zero, conditionally moves `value` to `self` when `condition` is
    /// equal to zero.
    fn cmovnz(&mut self, value: Self, condition: Condition);
}

impl Cmov for u8 {
    #[inline(always)]
    fn cmovz(&mut self, value: Self, condition: Condition) {
        let mut tmp = *self as u16;
        tmp.cmovz(value as u16, condition);
        *self = tmp as u8;
    }

    #[inline(always)]
    fn cmovnz(&mut self, value: Self, condition: Condition) {
        let mut tmp = *self as u16;
        tmp.cmovnz(value as u16, condition);
        *self = tmp as u8;
    }
}

impl Cmov for u128 {
    #[inline(always)]
    fn cmovz(&mut self, value: Self, condition: Condition) {
        let mut lo = (*self & u64::MAX as u128) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovz((value & u64::MAX as u128) as u64, condition);
        hi.cmovz((value >> 64) as u64, condition);

        *self = (lo as u128) | (hi as u128) << 64;
    }

    #[inline(always)]
    fn cmovnz(&mut self, value: Self, condition: Condition) {
        let mut lo = (*self & u64::MAX as u128) as u64;
        let mut hi = (*self >> 64) as u64;

        lo.cmovnz((value & u64::MAX as u128) as u64, condition);
        hi.cmovnz((value >> 64) as u64, condition);

        *self = (lo as u128) | (hi as u128) << 64;
    }
}
