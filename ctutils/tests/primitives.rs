//! Differential tests against the core integer operations.

use ctutils::{Choice, primitives::*};
use proptest::prelude::*;

proptest! {
    #[test]
    fn sub_u32(lhs in any::<u32>(), rhs in any::<u32>(), incoming in any::<bool>()) {
        let (expected, first_borrow) = lhs.overflowing_sub(rhs);
        let (expected, second_borrow) = expected.overflowing_sub(u32::from(incoming));
        let (difference, borrow) = borrowing_sub_u32(lhs, rhs, Choice::from_u8_lsb(u8::from(incoming)));
        prop_assert_eq!((difference, borrow.to_bool()), (expected, first_borrow | second_borrow));
    }

    #[test]
    fn sub_u64(lhs in any::<u64>(), rhs in any::<u64>(), incoming in any::<bool>()) {
        let (expected, first_borrow) = lhs.overflowing_sub(rhs);
        let (expected, second_borrow) = expected.overflowing_sub(u64::from(incoming));
        let (difference, borrow) = borrowing_sub_u64(lhs, rhs, Choice::from_u8_lsb(u8::from(incoming)));
        prop_assert_eq!((difference, borrow.to_bool()), (expected, first_borrow | second_borrow));
    }

    #[test]
    fn sub_mask_u32(lhs in any::<u32>(), rhs in any::<u32>(), incoming in any::<u32>()) {
        let (expected, first_borrow) = lhs.overflowing_sub(rhs);
        let (expected, second_borrow) = expected.overflowing_sub(incoming >> 31);
        let expected_borrow = u32::from(first_borrow | second_borrow).wrapping_neg();
        prop_assert_eq!(borrowing_sub_mask_u32(lhs, rhs, incoming), (expected, expected_borrow));
    }

    #[test]
    fn sub_mask_u64(lhs in any::<u64>(), rhs in any::<u64>(), incoming in any::<u64>()) {
        let (expected, first_borrow) = lhs.overflowing_sub(rhs);
        let (expected, second_borrow) = expected.overflowing_sub(incoming >> 63);
        let expected_borrow = u64::from(first_borrow | second_borrow).wrapping_neg();
        prop_assert_eq!(borrowing_sub_mask_u64(lhs, rhs, incoming), (expected, expected_borrow));
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, reason = "extract the low limb")]
    fn sub_mask_u32_limbs(lhs in any::<u64>(), rhs in any::<u64>(), incoming in any::<u32>()) {
        let (expected, first_borrow) = lhs.overflowing_sub(rhs);
        let (expected, second_borrow) = expected.overflowing_sub(u64::from(incoming >> 31));
        let expected_borrow = u32::from(first_borrow | second_borrow).wrapping_neg();
        let (lo, borrow) = borrowing_sub_mask_u32(lhs as u32, rhs as u32, incoming);
        let (hi, borrow) = borrowing_sub_mask_u32((lhs >> 32) as u32, (rhs >> 32) as u32, borrow);
        let difference = (u64::from(hi) << 32) | u64::from(lo);
        prop_assert_eq!((difference, borrow), (expected, expected_borrow));
    }

    #[test]
    #[allow(clippy::cast_possible_truncation, reason = "extract the low limb")]
    fn sub_mask_u64_limbs(lhs in any::<u128>(), rhs in any::<u128>(), incoming in any::<u64>()) {
        let (expected, first_borrow) = lhs.overflowing_sub(rhs);
        let (expected, second_borrow) = expected.overflowing_sub(u128::from(incoming >> 63));
        let expected_borrow = u64::from(first_borrow | second_borrow).wrapping_neg();
        let (lo, borrow) = borrowing_sub_mask_u64(lhs as u64, rhs as u64, incoming);
        let (hi, borrow) = borrowing_sub_mask_u64((lhs >> 64) as u64, (rhs >> 64) as u64, borrow);
        let difference = (u128::from(hi) << 64) | u128::from(lo);
        prop_assert_eq!((difference, borrow), (expected, expected_borrow));
    }

    #[test]
    fn zero_counts_u32(value in any::<u32>()) {
        prop_assert_eq!(leading_zeros_u32(value), value.leading_zeros());
        prop_assert_eq!(trailing_zeros_u32(value), value.trailing_zeros());
    }

    #[test]
    fn zero_counts_u64(value in any::<u64>()) {
        prop_assert_eq!(leading_zeros_u64(value), value.leading_zeros());
        prop_assert_eq!(trailing_zeros_u64(value), value.trailing_zeros());
    }

    #[test]
    fn add_u64(lhs in any::<u64>(), rhs in any::<u64>()) {
        let (sum, carry) = overflowing_add_u64(lhs, rhs);
        prop_assert_eq!((sum, carry.to_bool()), lhs.overflowing_add(rhs));
    }

    #[test]
    fn add_u128(lhs in any::<u128>(), rhs in any::<u128>()) {
        let (sum, carry) = overflowing_add_u128(lhs, rhs);
        prop_assert_eq!((sum, carry.to_bool()), lhs.overflowing_add(rhs));
    }
}
