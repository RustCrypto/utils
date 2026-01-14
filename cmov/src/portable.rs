//! Portable "best effort" implementation of `Cmov`.
//!
//! This implementation is based on portable bitwise arithmetic but cannot
//! guarantee that the resulting generated assembly is free of branch
//! instructions.

// TODO(tarcieri): more optimized implementation for small integers

use crate::{Cmov, CmovEq, Condition};

// Use `asm!` on architectures where it's stable but we don't have a custom-written backend
#[cfg(all(
    not(miri),
    any(
        target_arch = "arm",
        target_arch = "arm64ec",
        target_arch = "loongarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "s390x"
    )
))]
fn black_box<T: Copy>(val: T) -> T {
    #[allow(trivial_casts)]
    unsafe {
        core::arch::asm!(
            "# {}",
            in(reg) &val as *const T as *const (),
            options(readonly, preserves_flags, nostack),
        );
    }
    val
}

// Use `black_box` as a portable fallback for other architectures
#[cfg(not(all(
    not(miri),
    any(
        target_arch = "arm",
        target_arch = "arm64ec",
        target_arch = "loongarch64",
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "s390x"
    )
)))]
use core::hint::black_box;

/// Bitwise non-zero: returns `1` if `x != 0`, and otherwise returns `0`.
macro_rules! bitnz {
    ($value:expr, $bits:expr) => {
        black_box(($value | $value.wrapping_neg()) >> ($bits - 1))
    };
}

impl Cmov for u16 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovnz(&(*value as u64), condition);
        *self = tmp as u16;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovz(&(*value as u64), condition);
        *self = tmp as u16;
    }
}

impl CmovEq for u16 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmovne(&(*rhs as u64), input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmoveq(&(*rhs as u64), input, output);
    }
}

impl Cmov for u32 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovnz(&(*value as u64), condition);
        *self = tmp as u32;
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mut tmp = *self as u64;
        tmp.cmovz(&(*value as u64), condition);
        *self = tmp as u32;
    }
}

impl CmovEq for u32 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmovne(&(*rhs as u64), input, output);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        (*self as u64).cmoveq(&(*rhs as u64), input, output);
    }
}

impl Cmov for u64 {
    #[inline]
    fn cmovnz(&mut self, value: &Self, condition: Condition) {
        let mask = (bitnz!(condition, u8::BITS) as u64).wrapping_sub(1);
        *self = (*self & mask) | (*value & !mask);
    }

    #[inline]
    fn cmovz(&mut self, value: &Self, condition: Condition) {
        let mask = (1 ^ bitnz!(condition, u8::BITS) as u64).wrapping_sub(1);
        *self = (*self & mask) | (*value & !mask);
    }
}

impl CmovEq for u64 {
    #[inline]
    fn cmovne(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u64::BITS) as u8;
        output.cmovnz(&input, ne);
    }

    #[inline]
    fn cmoveq(&self, rhs: &Self, input: Condition, output: &mut Condition) {
        let ne = bitnz!(self ^ rhs, u64::BITS) as u8;
        output.cmovnz(&input, ne ^ 1);
    }
}
