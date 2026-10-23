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
