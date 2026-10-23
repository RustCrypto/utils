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

macro_rules! subtraction_checks {
    ($module:ident, $ty:ty, $sub:ident, $mask:ident) => {
        mod $module {
            use super::*;

            #[checkct]
            pub fn borrowing_sub() {
                black_box(PublicRng.next_u32());
                let incoming = Choice::from_u8_lsb(private_integer!(u8));
                let (difference, borrow) =
                    primitives::$sub(private_integer!($ty), private_integer!($ty), incoming);
                black_box((difference, borrow.to_u8()));
            }

            #[checkct]
            pub fn borrowing_sub_zero_borrow() {
                black_box(PublicRng.next_u32());
                let (difference, borrow) =
                    primitives::$sub(private_integer!($ty), private_integer!($ty), Choice::FALSE);
                black_box((difference, borrow.to_u8()));
            }

            #[checkct]
            pub fn borrowing_sub_limbs() {
                black_box(PublicRng.next_u32());
                let lhs = [private_integer!($ty), private_integer!($ty)];
                let rhs = [private_integer!($ty), private_integer!($ty)];
                let mut borrow = Choice::from_u8_lsb(private_integer!(u8));
                let mut difference = [0; 2];
                for i in 0..2 {
                    (difference[i], borrow) = primitives::$sub(lhs[i], rhs[i], borrow);
                }
                black_box((difference, borrow.to_u8()));
            }

            /// Division conditionally adds back the divisor using the outgoing borrow mask.
            #[checkct]
            pub fn borrowing_sub_add_back() {
                black_box(PublicRng.next_u32());
                let lhs = private_integer!($ty);
                let rhs = private_integer!($ty);
                let incoming = Choice::from_u8_lsb(private_integer!(u8));
                let (difference, borrow) = primitives::$sub(lhs, rhs, incoming);
                black_box(difference.wrapping_add(rhs & borrow.$mask()));
            }
        }
    };
}

subtraction_checks!(u32_sub, u32, borrowing_sub_u32, to_u32_mask);
subtraction_checks!(u64_sub, u64, borrowing_sub_u64, to_u64_mask);
