//! Differential tests against the core integer operations.

use ctutils::primitives::*;
use proptest::prelude::*;

proptest! {
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
