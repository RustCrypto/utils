//! Constant-time integer primitives for cryptographic arithmetic.
//!
//! Core integer methods can lower to branches or table lookups on targets without the corresponding
//! instructions. These helpers provide alternatives if `value` is zero-bit counting  and carry propagation.
//!
//! Like [`Choice`], they use optimization barriers where needed. Their generated code should still
//! be checked for the target and compiler in use.
//!
//! Native zero counts are used when supported and advertised by the target configuration.
use crate::Choice;

/// Count leading zero bits, returning 32 if `value` is zero.
#[inline]
#[must_use]
pub const fn leading_zeros_u32(value: u32) -> u32 {
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64",
        target_feature = "thumb2",
        target_feature = "zbb"
    ))]
    {
        value.leading_zeros()
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64",
        target_feature = "thumb2",
        target_feature = "zbb"
    )))]
    {
        leading_zeros_u32_soft(value)
    }
}

/// Count trailing zero bits, returning 32 if `value` is zero.
#[inline]
#[must_use]
pub const fn trailing_zeros_u32(value: u32) -> u32 {
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64",
        target_feature = "thumb2",
        target_feature = "zbb"
    ))]
    {
        value.trailing_zeros()
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64",
        target_feature = "thumb2",
        target_feature = "zbb"
    )))]
    {
        // Reversing bits uses fixed shifts and masks when RBIT is unavailable.
        leading_zeros_u32_soft(value.reverse_bits())
    }
}

/// Count leading zero bits, returning 64 if `value` is zero.
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn leading_zeros_u64(value: u64) -> u32 {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        value.leading_zeros()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        let hi = (value >> 32) as u32;
        let lo = value as u32;
        let mask = Choice::from_u32_nz(hi).not().to_u32_mask();
        leading_zeros_u32(hi).wrapping_add(mask & leading_zeros_u32(lo))
    }
}

/// Count trailing zero bits, returning 64 if `value` is zero.
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn trailing_zeros_u64(value: u64) -> u32 {
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        value.trailing_zeros()
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        let hi = (value >> 32) as u32;
        let lo = value as u32;
        let mask = Choice::from_u32_nz(lo).not().to_u32_mask();
        trailing_zeros_u32(lo).wrapping_add(mask & trailing_zeros_u32(hi))
    }
}

/// Fixed-stage binary search for the highest set bit, including the zero case.
#[cfg(any(
    test,
    not(any(
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "x86_64",
        target_feature = "thumb2",
        target_feature = "zbb"
    ))
))]
#[inline]
#[allow(clippy::arithmetic_side_effects, reason = "shift is between 1 and 16")]
const fn leading_zeros_u32_soft(mut value: u32) -> u32 {
    let mut count = 0u32;
    let mut shift = 16;
    while shift != 0 {
        let mask = Choice::from_u32_nz(value >> (u32::BITS - shift))
            .not()
            .to_u32_mask();
        value ^= mask & (value ^ (value << shift));
        count = count.wrapping_add(mask & shift);
        shift >>= 1;
    }
    count.wrapping_add(Choice::from_u32_nz(value).not().to_u32_mask() & 1)
}

/// Add two integers, returning the wrapping sum and [`Choice::TRUE`] on overflow.
#[inline]
#[must_use]
pub const fn overflowing_add_u64(lhs: u64, rhs: u64) -> (u64, Choice) {
    let sum = lhs.wrapping_add(rhs);
    let carry = ((lhs & rhs) | ((lhs | rhs) & !sum)) >> (u64::BITS - 1);
    (sum, Choice::from_u64_lsb(carry))
}

/// Add two integers, returning the wrapping sum and [`Choice::TRUE`] on overflow.
#[inline]
#[must_use]
pub const fn overflowing_add_u128(lhs: u128, rhs: u128) -> (u128, Choice) {
    #[cfg(target_arch = "riscv32")]
    {
        overflowing_add_u128_halves(lhs, rhs)
    }
    #[cfg(not(target_arch = "riscv32"))]
    {
        let sum = lhs.wrapping_add(rhs);
        let carry = ((lhs & rhs) | ((lhs | rhs) & !sum)) >> (u128::BITS - 1);
        (sum, Choice::from_u128_lsb(carry))
    }
}

// RV32 can branch even in `u128::wrapping_add`, before computing the final carry.
// Keep this workaround local to that target, preserving native addition elsewhere.
#[cfg(any(target_arch = "riscv32", test))]
#[inline]
#[allow(clippy::cast_possible_truncation)]
const fn overflowing_add_u128_halves(lhs: u128, rhs: u128) -> (u128, Choice) {
    let (lo, lo_carry) = overflowing_add_u64(lhs as u64, rhs as u64);
    let (hi, hi_carry) = overflowing_add_u64((lhs >> 64) as u64, (rhs >> 64) as u64);
    let (hi, carry) = overflowing_add_u64(hi, lo_carry.to_u8() as u64);
    (((hi as u128) << 64) | lo as u128, hi_carry.or(carry))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Keep all public entrypoints usable by const callers such as crypto-bigint.
    const _: () = {
        assert!(leading_zeros_u32(0) == 32);
        assert!(trailing_zeros_u32(0) == 32);
        assert!(leading_zeros_u64(0) == 64);
        assert!(trailing_zeros_u64(0) == 64);
        assert!(overflowing_add_u64(u64::MAX, 1).1.to_bool());
        assert!(overflowing_add_u128(u128::MAX, 1).1.to_bool());
    };

    #[test]
    fn zero_counts_u32() {
        for bit in 0..u32::BITS {
            for value in [0, 1 << bit, !(1 << bit), u32::MAX >> bit] {
                assert_eq!(leading_zeros_u32(value), value.leading_zeros());
                assert_eq!(trailing_zeros_u32(value), value.trailing_zeros());
                // Exercise the software fallback even on hosts with native instructions.
                assert_eq!(leading_zeros_u32_soft(value), value.leading_zeros());
                assert_eq!(
                    leading_zeros_u32_soft(value.reverse_bits()),
                    value.trailing_zeros()
                );
            }
        }
    }

    #[test]
    fn zero_counts_u64() {
        for bit in 0..u64::BITS {
            for value in [0, 1 << bit, !(1 << bit), u64::MAX >> bit] {
                assert_eq!(leading_zeros_u64(value), value.leading_zeros());
                assert_eq!(trailing_zeros_u64(value), value.trailing_zeros());
            }
        }
    }

    macro_rules! addition_tests {
        ($test:ident, $ty:ty, $add:ident) => {
            #[test]
            fn $test() {
                // Include both full-width and half-width carry boundaries.
                let half = 1 << (<$ty>::BITS >> 1);
                let high = 1 << (<$ty>::BITS - 1);
                let values = [0, 1, half - 1, half, half + 1, high - 1, high, <$ty>::MAX];
                for lhs in values {
                    for rhs in values {
                        let (sum, carry) = $add(lhs, rhs);
                        assert_eq!((sum, carry.to_bool()), lhs.overflowing_add(rhs));
                    }
                }
            }
        };
    }

    addition_tests!(add_u64, u64, overflowing_add_u64);
    addition_tests!(add_u128, u128, overflowing_add_u128);
    addition_tests!(add_u128_halves, u128, overflowing_add_u128_halves);
}
