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

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskne32(*self, *rhs) & 0xFF) as u8);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskeq32(*self, *rhs) & 0xFF) as u8);
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskne64(*self, *rhs) & 0xFF) as u8);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        *output = masksel(*output, input, (maskeq64(*self, *rhs) & 0xFF) as u8);
    }
}

/// Returns `u32::MAX` if `x` is equal to `y`, otherwise returns `0` (32-bit version)
#[inline]
fn maskeq32(x: u32, y: u32) -> u32 {
    !maskne32(x, y)
}

/// Returns `u32::MAX` if `x` is equal to `y`, otherwise returns `0` (64-bit version)
#[inline]
fn maskeq64(x: u64, y: u64) -> u64 {
    !maskne64(x, y)
}

/// Returns `0` if `x` is equal to `y`, otherwise returns `1` (32-bit version)
#[inline]
fn maskne32(x: u32, y: u32) -> u32 {
    masknz32(x ^ y)
}

/// Returns `0` if `x` is equal to `y`, otherwise returns `1` (64-bit version)
#[inline]
fn maskne64(x: u64, y: u64) -> u64 {
    masknz64(x ^ y)
}

/// Return a [`u32::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
#[cfg(not(target_arch = "arm"))]
fn masknz32(condition: u32) -> u32 {
    let x = condition | condition.wrapping_neg(); // MSB of `x` now `1` if non-zero
    let nz = core::hint::black_box(x >> (u32::BITS - 1)); // Extract MSB
    nz.wrapping_neg()
}

/// Return a [`u64::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
#[cfg(not(target_arch = "arm"))]
fn masknz64(condition: u64) -> u64 {
    let x = condition | condition.wrapping_neg(); // MSB of `x` now `1` if non-zero
    let nz = core::hint::black_box(x >> (u64::BITS - 1)); // Extract MSB
    nz.wrapping_neg()
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

#[cfg(test)]
mod tests {
    // Spot check up to a given limit
    const TEST_LIMIT: u32 = 65536;

    #[test]
    fn maskeq32() {
        assert_eq!(super::maskeq32(0, 0), u32::MAX);
        assert_eq!(super::maskeq32(1, 0), 0);
        assert_eq!(super::maskeq32(0, 1), 0);
        assert_eq!(super::maskeq32(1, 1), u32::MAX);
        assert_eq!(super::maskeq32(u32::MAX, 1), 0);
        assert_eq!(super::maskeq32(1, u32::MAX), 0);
        assert_eq!(super::maskeq32(u32::MAX, u32::MAX), u32::MAX);
    }

    #[test]
    fn maskeq64() {
        assert_eq!(super::maskeq64(0, 0), u64::MAX);
        assert_eq!(super::maskeq64(1, 0), 0);
        assert_eq!(super::maskeq64(0, 1), 0);
        assert_eq!(super::maskeq64(1, 1), u64::MAX);
        assert_eq!(super::maskeq64(u64::MAX, 1), 0);
        assert_eq!(super::maskeq64(1, u64::MAX), 0);
        assert_eq!(super::maskeq64(u64::MAX, u64::MAX), u64::MAX);
    }

    #[test]
    fn maskne32() {
        assert_eq!(super::maskne32(0, 0), 0);
        assert_eq!(super::maskne32(1, 0), u32::MAX);
        assert_eq!(super::maskne32(0, 1), u32::MAX);
        assert_eq!(super::maskne32(1, 1), 0);
        assert_eq!(super::maskne32(u32::MAX, 1), u32::MAX);
        assert_eq!(super::maskne32(1, u32::MAX), u32::MAX);
        assert_eq!(super::maskne32(u32::MAX, u32::MAX), 0);
    }

    #[test]
    fn maskne64() {
        assert_eq!(super::maskne64(0, 0), 0);
        assert_eq!(super::maskne64(1, 0), u64::MAX);
        assert_eq!(super::maskne64(0, 1), u64::MAX);
        assert_eq!(super::maskne64(1, 1), 0);
        assert_eq!(super::maskne64(u64::MAX, 1), u64::MAX);
        assert_eq!(super::maskne64(1, u64::MAX), u64::MAX);
        assert_eq!(super::maskne64(u64::MAX, u64::MAX), 0);
    }

    #[test]
    fn masknz32() {
        assert_eq!(super::masknz32(0), 0);
        for i in 1..=TEST_LIMIT {
            assert_eq!(super::masknz32(i), u32::MAX);
        }

        for i in (u32::MAX - TEST_LIMIT)..=u32::MAX {
            assert_eq!(super::masknz32(i), u32::MAX);
        }
    }

    #[test]
    fn masknz64() {
        assert_eq!(super::masknz64(0), 0);
        for i in 1..=(TEST_LIMIT as u64) {
            assert_eq!(super::masknz64(i), u64::MAX);
        }

        for i in (u64::MAX - TEST_LIMIT as u64)..=u64::MAX {
            assert_eq!(super::masknz64(i), u64::MAX);
        }
    }

    #[test]
    fn masksel() {
        assert_eq!(super::masksel(23u8, 42u8, 0u8), 23u8);
        assert_eq!(super::masksel(23u8, 42u8, u8::MAX), 42u8);

        assert_eq!(super::masksel(17u32, 101077u32, 0u32), 17u32);
        assert_eq!(super::masksel(17u32, 101077u32, u32::MAX), 101077u32);

        assert_eq!(super::masksel(129u64, 0xFFEEDDCCBBAA9988u64, 0u64), 129u64);
        assert_eq!(
            super::masksel(129u64, 0xFFEEDDCCBBAA9988u64, u64::MAX),
            0xFFEEDDCCBBAA9988u64
        );
    }
}
