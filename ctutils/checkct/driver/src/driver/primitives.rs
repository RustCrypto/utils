//! Regression cases for the primitives used by crypto-bigint.

use super::*;
use ctutils::primitives;

macro_rules! zero_count_checks {
    ($module:ident, $ty:ty, $leading:ident, $trailing:ident, $from_nz:ident) => {
        mod $module {
            use super::*;

            #[checkct]
            pub fn leading_zeros() {
                black_box(PublicRng.next_u32());
                black_box(primitives::$leading(private_integer!($ty)));
            }

            #[checkct]
            pub fn trailing_zeros() {
                black_box(PublicRng.next_u32());
                black_box(primitives::$trailing(private_integer!($ty)));
            }

            /// Do not put an out-of-line boundary around each limb's zero count:
            /// LLVM must not introduce branches when inlining it into the scan.
            #[checkct]
            pub fn leading_zeros_limbs() {
                black_box(PublicRng.next_u32());
                let limbs = [private_integer!($ty), private_integer!($ty)];
                let mut count = 0;
                let mut mask = u32::MAX;
                for limb in limbs.into_iter().rev() {
                    count += mask & primitives::$leading(limb);
                    mask &= Choice::$from_nz(limb).not().to_u32_mask();
                }
                black_box(count);
            }

            #[checkct]
            pub fn trailing_zeros_limbs() {
                black_box(PublicRng.next_u32());
                let limbs = [private_integer!($ty), private_integer!($ty)];
                let mut count = 0;
                let mut mask = u32::MAX;
                for limb in limbs {
                    count += mask & primitives::$trailing(limb);
                    mask &= Choice::$from_nz(limb).not().to_u32_mask();
                }
                black_box(count);
            }
        }
    };
}

zero_count_checks!(
    u32_bits,
    u32,
    leading_zeros_u32,
    trailing_zeros_u32,
    from_u32_nz
);
zero_count_checks!(
    u64_bits,
    u64,
    leading_zeros_u64,
    trailing_zeros_u64,
    from_u64_nz
);

macro_rules! addition_checks {
    ($module:ident, $ty:ty, $add:ident, $mask:ident) => {
        mod $module {
            use super::*;

            #[checkct]
            pub fn overflowing_add() {
                black_box(PublicRng.next_u32());
                let (sum, carry) = primitives::$add(private_integer!($ty), private_integer!($ty));
                black_box((sum, carry.to_u8()));
            }

            /// Montgomery multiplication shifts the carry into a wider accumulator.
            /// Exercise that use as well as returning the carry by itself.
            #[checkct]
            pub fn overflowing_add_accumulate_carry() {
                black_box(PublicRng.next_u32());
                let (sum, carry) = primitives::$add(private_integer!($ty), private_integer!($ty));
                let half_bits = <$ty>::BITS >> 1;
                black_box(((carry.$mask() & 1) << half_bits) | (sum >> half_bits));
            }
        }
    };
}

addition_checks!(u64_add, u64, overflowing_add_u64, to_u64_mask);
addition_checks!(u128_add, u128, overflowing_add_u128, to_u128_mask);
