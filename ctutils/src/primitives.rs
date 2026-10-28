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
use core::hint::black_box;

/// Bitwise less-than-or equal: returns `1` if `x <= y`, and otherwise returns `0`.
///
/// See "Hacker's Delight" 2nd edition, section 2-12 (Comparison predicates)
macro_rules! bitle {
    ($x:expr, $y:expr, $bits:expr) => {
        (((!$x) | $y) & (($x ^ $y) | !($y.wrapping_sub($x)))) >> ($bits - 1)
    };
}

/// Outgoing borrow bit for a wrapping `x - y - borrow_in` result.
///
/// See "Hacker's Delight", 2nd edition, section 2-13 (Overflow detection)
macro_rules! bitborrow {
    ($x:expr, $y:expr, $result:expr, $bits:expr) => {
        (((!$x) & $y) | (((!$x) | $y) & $result)) >> ($bits - 1)
    };
}

/// Bitwise less-than: returns `1` if `x < y`, and otherwise returns `0`.
///
/// See "Hacker's Delight" 2nd edition, section 2-12 (Comparison predicates)
macro_rules! bitlt {
    ($x:expr, $y:expr, $bits:expr) => {
        $crate::primitives::bitborrow!($x, $y, ($x).wrapping_sub($y), $bits)
    };
}

/// Bitwise non-zero: returns `1` if `x != 0`, and otherwise returns `0`.
macro_rules! bitnz {
    ($value:expr, $bits:expr) => {
        ($value | $value.wrapping_neg()) >> ($bits - 1)
    };
}

pub(crate) use {bitborrow, bitle, bitlt, bitnz};

// Keep the comparison bit opaque before narrowing it to Choice's byte representation.
// ARM targets without pointer-width atomics cover Thumb-1 on Rust 1.87, which does not
// advertise `thumb2` as a target feature. The other targets retain the original lowering.
macro_rules! comparison_primitives {
    ($ty:ty, $le:ident, $lt:ident, $nz:ident, $from_lsb:ident) => {
        #[inline(always)]
        pub(crate) const fn $le(lhs: $ty, rhs: $ty) -> Choice {
            let bit = bitle!(lhs, rhs, <$ty>::BITS);
            #[cfg(all(target_arch = "arm", not(target_has_atomic = "ptr")))]
            let bit = black_box(bit);
            Choice::$from_lsb(bit)
        }

        #[cfg_attr(
            all(target_arch = "arm", not(target_has_atomic = "ptr")),
            inline(never)
        )]
        #[cfg_attr(
            not(all(target_arch = "arm", not(target_has_atomic = "ptr"))),
            inline(always)
        )]
        pub(crate) const fn $lt(lhs: $ty, rhs: $ty) -> Choice {
            let bit = $crate::primitives::bitlt!(lhs, rhs, <$ty>::BITS);
            #[cfg(all(target_arch = "arm", not(target_has_atomic = "ptr")))]
            let bit = black_box(bit);
            Choice::$from_lsb(bit)
        }

        #[inline(always)]
        pub(crate) const fn $nz(value: $ty) -> Choice {
            let bit = bitnz!(value, <$ty>::BITS);
            #[cfg(all(target_arch = "arm", not(target_has_atomic = "ptr")))]
            let bit = black_box(bit);
            Choice::$from_lsb(bit)
        }
    };
}

comparison_primitives!(u32, le_u32, lt_u32, nz_u32, from_u32_lsb);
comparison_primitives!(u64, le_u64, lt_u64, nz_u64, from_u64_lsb);

macro_rules! borrowing_sub_primitives {
    ($ty:ty, $sub:ident, $sub_mask:ident, $sub_inner:ident, $from_lsb:ident) => {
        /// Subtract `rhs` and an incoming borrow from `lhs`.
        ///
        /// Returns the wrapping difference and [`Choice::TRUE`] if the subtraction underflows.
        /// The incoming `borrow` subtracts one when true and zero when false.
        #[inline]
        #[must_use]
        pub const fn $sub(lhs: $ty, rhs: $ty, borrow: Choice) -> ($ty, Choice) {
            let (difference, borrow) = $sub_inner(lhs, rhs, borrow.to_u8() as $ty);
            (difference, Choice::$from_lsb(borrow))
        }

        /// Subtract `rhs` and an incoming borrow from `lhs`, using word-sized borrow masks.
        ///
        /// The incoming borrow is the most significant bit of `borrow`; all other bits are ignored.
        /// Returns the wrapping difference and an all-ones mask if the subtraction underflows,
        /// or zero otherwise.
        ///
        /// The outgoing mask can be passed directly to the next limb's subtraction, avoiding
        /// conversions to and from [`Choice`] at each step of a multi-limb subtraction.
        #[inline]
        #[must_use]
        pub const fn $sub_mask(lhs: $ty, rhs: $ty, borrow: $ty) -> ($ty, $ty) {
            let (difference, borrow) = $sub_inner(lhs, rhs, borrow >> (<$ty>::BITS - 1));
            (difference, borrow.wrapping_neg())
        }

        // The incoming borrow must be zero or one; the outgoing borrow is also zero or one.
        #[inline(always)]
        const fn $sub_inner(lhs: $ty, rhs: $ty, borrow: $ty) -> ($ty, $ty) {
            let difference = lhs.wrapping_sub(rhs).wrapping_sub(borrow);
            // Keep all arithmetic in the operand type: widening can trigger incorrect
            // optimization on older compilers: https://github.com/rust-lang/rust/issues/149522
            // Keep the full-width bit opaque before forming a mask or narrowing it to Choice,
            // so a caller's masked add-back cannot be folded into a secret-dependent branch.
            let borrow = black_box(bitborrow!(lhs, rhs, difference, <$ty>::BITS));
            (difference, borrow)
        }
    };
}

borrowing_sub_primitives!(
    u32,
    borrowing_sub_u32,
    borrowing_sub_mask_u32,
    borrowing_sub_inner_u32,
    from_u32_lsb
);
borrowing_sub_primitives!(
    u64,
    borrowing_sub_u64,
    borrowing_sub_mask_u64,
    borrowing_sub_inner_u64,
    from_u64_lsb
);

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
        assert!(borrowing_sub_u32(0, 0, Choice::TRUE).0 == u32::MAX);
        assert!(borrowing_sub_u32(0, 0, Choice::TRUE).1.to_bool());
        assert!(borrowing_sub_u64(0, 0, Choice::TRUE).0 == u64::MAX);
        assert!(borrowing_sub_u64(0, 0, Choice::TRUE).1.to_bool());
        assert!(!borrowing_sub_u32(1, 0, Choice::TRUE).1.to_bool());
        assert!(!borrowing_sub_u64(1, 0, Choice::TRUE).1.to_bool());
        assert!(borrowing_sub_mask_u32(0, 0, u32::MAX).0 == u32::MAX);
        assert!(borrowing_sub_mask_u32(0, 0, u32::MAX).1 == u32::MAX);
        assert!(borrowing_sub_mask_u64(0, 0, u64::MAX).0 == u64::MAX);
        assert!(borrowing_sub_mask_u64(0, 0, u64::MAX).1 == u64::MAX);
        assert!(borrowing_sub_mask_u32(1, 0, u32::MAX).1 == 0);
        assert!(borrowing_sub_mask_u64(1, 0, u64::MAX).1 == 0);
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

    macro_rules! subtraction_tests {
        ($test:ident, $ty:ty, $sub:ident, $sub_mask:ident) => {
            #[test]
            fn $test() {
                let half = 1 << (<$ty>::BITS >> 1);
                let high = 1 << (<$ty>::BITS - 1);
                let values = [0, 1, half - 1, half, half + 1, high - 1, high, <$ty>::MAX];
                for lhs in values {
                    for rhs in values {
                        // Include noncanonical masks: only the most significant bit matters.
                        for mask in [0, 1, high - 1, high, <$ty>::MAX - 1, <$ty>::MAX] {
                            let incoming = mask >> (<$ty>::BITS - 1) != 0;
                            let (expected, first_borrow) = lhs.overflowing_sub(rhs);
                            let (expected, second_borrow) =
                                expected.overflowing_sub(<$ty>::from(incoming));
                            let (difference, borrow) =
                                $sub(lhs, rhs, Choice::from_u8_lsb(u8::from(incoming)));
                            assert_eq!(
                                (difference, borrow.to_bool()),
                                (expected, first_borrow | second_borrow)
                            );
                            assert_eq!(
                                $sub_mask(lhs, rhs, mask),
                                (
                                    expected,
                                    <$ty>::from(first_borrow | second_borrow).wrapping_neg()
                                )
                            );
                        }
                    }
                }
            }
        };
    }

    subtraction_tests!(sub_u32, u32, borrowing_sub_u32, borrowing_sub_mask_u32);
    subtraction_tests!(sub_u64, u64, borrowing_sub_u64, borrowing_sub_mask_u64);

    #[test]
    fn sub_mask_propagates_borrow() {
        let (lo, borrow) = borrowing_sub_mask_u32(0, 1, 0);
        let (mid, borrow) = borrowing_sub_mask_u32(0, 0, borrow);
        assert_eq!((lo, mid, borrow), (u32::MAX, u32::MAX, u32::MAX));
        assert_eq!(borrowing_sub_mask_u32(1, 0, borrow), (0, 0));

        let (lo, borrow) = borrowing_sub_mask_u64(0, 1, 0);
        let (mid, borrow) = borrowing_sub_mask_u64(0, 0, borrow);
        assert_eq!((lo, mid, borrow), (u64::MAX, u64::MAX, u64::MAX));
        assert_eq!(borrowing_sub_mask_u64(1, 0, borrow), (0, 0));
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
