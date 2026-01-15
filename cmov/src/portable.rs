//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot guarantee that the
//! resulting generated assembly is free of branch instructions.
//!
//! For select platforms we use `asm!` for mask generation which should largely mitigate the
//! optimizer potentially inserting branches.

use crate::{Cmov, CmovEq, Condition};

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
        let mask = masknz32(condition);
        *self = (*self & !mask) | (*value & mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = masknz32(condition);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = testne32(*self, *rhs);
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let eq = testeq32(*self, *rhs);
        output.cmovnz(&input, eq);
    }
}

impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = masknz64(condition);
        *self = (*self & !mask) | (*value & mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = masknz64(condition);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = testne64(*self, *rhs);
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let eq = testeq64(*self, *rhs);
        output.cmovnz(&input, eq);
    }
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
    x |= x.wrapping_neg(); // MSB now set if non-zero
    core::hint::black_box(x >> (u64::BITS - 1)) // Extract MSB
}

/// Return a [`u32::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
#[cfg(not(target_arch = "arm"))]
fn masknz32(condition: Condition) -> u32 {
    testnz32(condition.into()).wrapping_neg()
}

/// Return a [`u64::MAX`] mask if `condition` is non-zero, otherwise return zero for a zero input.
#[cfg(not(target_arch = "arm"))]
fn masknz64(condition: Condition) -> u64 {
    testnz64(condition.into()).wrapping_neg()
}

/// Optimized mask generation for ARM32 targets.
#[cfg(target_arch = "arm")]
fn masknz32(condition: u8) -> u32 {
    let mut out = condition as u32;
    unsafe {
        core::arch::asm!(
        "rsbs {0}, {0}, #0",  // Reverse subtract
        "sbcs {0}, {0}, {0}", // Subtract with carry, setting flags
        inout(reg) out,
        options(nostack, nomem),
        );
    }
    out
}

/// 64-bit wrapper for targets that implement 32-bit mask generation in assembly.
#[cfg(target_arch = "arm")]
fn masknz64(condition: u8) -> u64 {
    let mask = masknz32(condition) as u64;
    mask | mask << 32
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn masknz32() {
        assert_eq!(super::masknz32(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::masknz32(i), u32::MAX);
        }
    }

    #[test]
    fn masknz64() {
        assert_eq!(super::masknz64(0), 0);
        for i in 1..=u8::MAX {
            assert_eq!(super::masknz64(i), u64::MAX);
        }
    }
}
