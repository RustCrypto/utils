#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
use core::arch::asm;

/// Move if zero.
///
/// Uses a `test` instruction to check if the given `condition` value is
/// equal to zero, then calls `cmovz` (a.k.a. `cmove`) to conditionally move
/// `src` to `dst` when `condition` is equal to zero.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn cmovz(condition: usize, src: usize, dst: &mut usize) {
    unsafe {
        asm! {
            "test {0}, {0}",
            "cmovz {1}, {2}",
            in(reg) condition,
            inlateout(reg) *dst,
            in(reg) src,
            options(pure, nomem, nostack),
        };
    }
}

/// Move if zero.
///
/// Uses a `cmp` instruction to check if the given `condition` value is
/// equal to zero, then calls `csel` to conditionally move
/// `src` to `dst` when `condition` is equal to zero.
#[cfg(any(target_arch = "aarch64"))]
#[inline(always)]
pub fn cmovz(condition: usize, src: usize, dst: &mut usize) {
    unsafe {
        asm! {
            "cmp {0}, 0",
            "csel {1}, {2}, {3}, EQ",
            in(reg) condition,
            inlateout(reg) *dst,
            in(reg) src,
            in(reg) *dst,
            options(pure, nomem, nostack),
        };
    }
}

/// Move if not zero.
///
/// Uses a `cmp` instruction to check if the given `condition` value is not
/// equal to zero, then calls `csel` to conditionally move
/// `src` to `dst` when `condition` is nonzero.
#[cfg(any(target_arch = "aarch64"))]
#[inline(always)]
pub fn cmovnz(condition: usize, src: usize, dst: &mut usize) {
    unsafe {
        asm! {
            "cmp {0}, 0",
            "csel {1}, {2}, {3}, NE",
            in(reg) condition,
            inlateout(reg) *dst,
            in(reg) src,
            in(reg) *dst,
            options(pure, nomem, nostack),
        };
    }
}

/// Move if not zero.
///
/// Uses a `test` instruction to check if the given `condition` value is not
/// equal to zero, then calls `cmovnz` (a.k.a. `cmovne`) to conditionally move
/// `src` to `dst` when `condition` is nonzero.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn cmovnz(condition: usize, src: usize, dst: &mut usize) {
    unsafe {
        asm! {
            "test {0}, {0}",
            "cmovnz {1}, {2}",
            in(reg) condition,
            inlateout(reg) *dst,
            in(reg) src,
            options(pure, nomem, nostack),
        };
    }
}

/// Move if zero (portable fallback implementation).
///
/// This implementation is based on portable bitwise arithmetic but cannot
/// guarantee that the resulting generated assembly is free of branch
/// instructions.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(never)]
pub fn cmovz(condition: usize, src: usize, dst: &mut usize) {
    let mask = (1 ^ is_non_zero(condition)).wrapping_sub(1);
    *dst = (*dst & mask) | (src & !mask);
}

/// Move if not zero (portable fallback implementation).
///
/// This implementation is based on portable bitwise arithmetic but cannot
/// guarantee that the resulting generated assembly is free of branch
/// instructions.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(never)]
pub fn cmovnz(condition: usize, src: usize, dst: &mut usize) {
    let mask = is_non_zero(condition).wrapping_sub(1);
    *dst = (*dst & mask) | (src & !mask);
}

/// Check if the given condition value is non-zero
///
/// # Returns
/// - `condition` is zero: `0`
/// - `condition` is non-zero: `1`
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
#[inline(always)]
fn is_non_zero(condition: usize) -> usize {
    const SHIFT_BITS: usize = core::mem::size_of::<usize>() - 1;
    ((condition | (!condition).wrapping_add(1)) >> SHIFT_BITS) & 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmovz_works() {
        let mut n = 24;
        cmovz(42, 42, &mut n);
        assert_eq!(n, 24);
        cmovz(0, 42, &mut n);
        assert_eq!(n, 42);
    }

    #[test]
    fn cmovnz_works() {
        let mut n = 24;
        cmovnz(0, 42, &mut n);
        assert_eq!(n, 24);
        cmovnz(42, 42, &mut n);
        assert_eq!(n, 42);
    }
}
