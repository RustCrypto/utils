//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot guarantee that the
//! resulting generated assembly is free of branch instructions.
//!
//! For select platforms we use `asm!` for mask generation which should largely mitigate the
//! optimizer potentially inserting branches.

use crate::{Cmov, CmovEq, Condition};
use core::ops::{BitAnd, BitOr, Not};

// Uses `Cmov` impl for `u32`
impl Cmov for u16 {
    #[inline]
    fn cmovnz(&mut self, value: &u16, condition: Condition) {
        let mut tmp = u32::from(*self);
        tmp.cmovnz(&(*value).into(), condition);
        *self = (tmp & 0xFFFF) as u16;
    }

    #[inline]
    fn cmovz(&mut self, value: &u16, condition: Condition) {
        let mut tmp = u32::from(*self);
        tmp.cmovz(&(*value).into(), condition);
        *self = (tmp & 0xFFFF) as u16;
    }
}

// Uses `CmovEq` impl for `u32`
impl CmovEq for u16 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        u32::from(*self).cmovne(&(*rhs).into(), input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        u32::from(*self).cmoveq(&(*rhs).into(), input, output);
    }
}

impl Cmov for u32 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, masknz32(condition.into()));
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, !masknz32(condition.into()));
    }
}

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        output.cmovnz(&input, testne32(*self, *rhs));
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        output.cmovnz(&input, testeq32(*self, *rhs));
    }
}

impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, masknz64(condition.into()));
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        *self = masksel(*self, *value, !masknz64(condition.into()));
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        output.cmovnz(&input, testne64(*self, *rhs));
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        output.cmovnz(&input, testeq64(*self, *rhs));
    }
}

/// Return a [`u32::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
#[cfg(not(target_arch = "arm"))]
fn masknz32(condition: u32) -> u32 {
    testnz32(condition).wrapping_neg()
}

/// Return a [`u64::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
#[cfg(not(target_arch = "arm"))]
fn masknz64(condition: u64) -> u64 {
    testnz64(condition).wrapping_neg()
}

/// Optimized mask generation for ARM32 targets.
///
/// This is written in assembly both for performance and because we've had problematic code
/// generation in this routine in the past which lead to the insertion of a branch, which using
/// assembly should guarantee won't happen again in the future (CVE-2026-23519).
#[cfg(target_arch = "arm")]
fn masknz32(condition: u32) -> u32 {
    let mut mask = condition;
    unsafe {
        core::arch::asm!(
        "rsbs {0}, {0}, #0",  // Reverse subtract
        "sbcs {0}, {0}, {0}", // Subtract with carry, setting flags
        inout(reg) mask,
        options(nostack, nomem),
        );
    }
    mask
}

/// 64-bit wrapper for targets that implement 32-bit mask generation in assembly.
#[cfg(target_arch = "arm")]
fn masknz64(condition: u64) -> u64 {
    let lo = masknz32((condition & 0xFFFF_FFFF) as u32);
    let hi = masknz32((condition >> 32) as u32);
    let mask = (lo | hi) as u64;
    mask | mask << 32
}

/// Given a supplied mask of `0` or all 1-bits (i.e. `u*::MAX`), select `a` if the mask is all-zeros
/// and `b` if the mask is all-ones.
///
/// This function shouldn't be used with a mask that isn't `0` or `u*::MAX`.
#[inline]
fn masksel<T>(a: T, b: T, mask: T) -> T
where
    T: BitAnd<Output = T> + BitOr<Output = T> + Copy + Not<Output = T>,
{
    (a & !mask) | (b & mask)
}

/// Returns `1` if `x` is equal to `y`, otherwise returns `0` (32-bit version)
fn testeq32(x: u32, y: u32) -> Condition {
    testne32(x, y) ^ 1
}

/// Returns `1` if `x` is equal to `y`, otherwise returns `0` (64-bit version)
fn testeq64(x: u64, y: u64) -> Condition {
    testne64(x, y) ^ 1
}

/// Returns `0` if `x` is equal to `y`, otherwise returns `1` (32-bit version)
fn testne32(x: u32, y: u32) -> Condition {
    (testnz32(x ^ y) & 0xFF) as Condition
}

/// Returns `0` if `x` is equal to `y`, otherwise returns `1` (64-bit version)
fn testne64(x: u64, y: u64) -> Condition {
    (testnz64(x ^ y) & 0xFF) as Condition
}

/// Returns `0` if `x` is `0`, otherwise returns `1` (32-bit version)
fn testnz32(mut x: u32) -> u32 {
    x |= x.wrapping_neg(); // MSB now set if non-zero
    core::hint::black_box(x >> (u32::BITS - 1)) // Extract MSB
}

/// Returns `0` if `x` is `0`, otherwise returns `1` (64-bit version)
fn testnz64(mut x: u64) -> u64 {
    x |= x.wrapping_neg(); // MSB now set if non-zero (or unset if zero)
    core::hint::black_box(x >> (u64::BITS - 1)) // Extract MSB
}

#[cfg(test)]
mod tests {
    #[test]
    fn masknz32() {
        assert_eq!(super::masknz32(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::masknz32(i.into()), u32::MAX);
        }
    }

    #[test]
    fn masknz64() {
        assert_eq!(super::masknz64(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::masknz64(i.into()), u64::MAX);
        }
    }

    #[test]
    fn masksel() {
        assert_eq!(super::masksel(23u8, 42u8, 0u8), 23u8);
        assert_eq!(super::masksel(23u8, 42u8, u8::MAX), 42u8);

        assert_eq!(super::masksel(17u32, 101077u32, 0u32), 17u32);
        assert_eq!(super::masksel(17u32, 101077u32, u32::MAX), 101077u32);
    }

    #[test]
    fn testeq32() {
        assert_eq!(super::testeq32(0, 0), 1);
        assert_eq!(super::testeq32(1, 0), 0);
        assert_eq!(super::testeq32(0, 1), 0);
        assert_eq!(super::testeq32(1, 1), 1);
        assert_eq!(super::testeq32(u32::MAX, 1), 0);
        assert_eq!(super::testeq32(1, u32::MAX), 0);
        assert_eq!(super::testeq32(u32::MAX, u32::MAX), 1);
    }

    #[test]
    fn testeq64() {
        assert_eq!(super::testeq64(0, 0), 1);
        assert_eq!(super::testeq64(1, 0), 0);
        assert_eq!(super::testeq64(0, 1), 0);
        assert_eq!(super::testeq64(1, 1), 1);
        assert_eq!(super::testeq64(u64::MAX, 1), 0);
        assert_eq!(super::testeq64(1, u64::MAX), 0);
        assert_eq!(super::testeq64(u64::MAX, u64::MAX), 1);
    }

    #[test]
    fn testne32() {
        assert_eq!(super::testne32(0, 0), 0);
        assert_eq!(super::testne32(1, 0), 1);
        assert_eq!(super::testne32(0, 1), 1);
        assert_eq!(super::testne32(1, 1), 0);
        assert_eq!(super::testne32(u32::MAX, 1), 1);
        assert_eq!(super::testne32(1, u32::MAX), 1);
        assert_eq!(super::testne32(u32::MAX, u32::MAX), 0);
    }

    #[test]
    fn testne64() {
        assert_eq!(super::testne64(0, 0), 0);
        assert_eq!(super::testne64(1, 0), 1);
        assert_eq!(super::testne64(0, 1), 1);
        assert_eq!(super::testne64(1, 1), 0);
        assert_eq!(super::testne64(u64::MAX, 1), 1);
        assert_eq!(super::testne64(1, u64::MAX), 1);
        assert_eq!(super::testne64(u64::MAX, u64::MAX), 0);
    }

    #[test]
    fn testnz32() {
        assert_eq!(super::testnz32(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::testnz32(i as u32), 1);
        }
    }

    #[test]
    fn testnz64() {
        assert_eq!(super::testnz64(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::testnz64(i as u64), 1);
        }
    }
}
