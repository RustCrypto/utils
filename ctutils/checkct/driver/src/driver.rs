use crate::rng::{PrivateRng, PublicRng};
use checkct_macros::checkct;
use core::hint::black_box;
use ctutils::{Choice, CtSelect};

macro_rules! private_integer {
    ($ty:ty) => {{
        let mut bytes = [0u8; size_of::<$ty>()];
        PrivateRng.fill_bytes(&mut bytes);
        <$ty>::from_le_bytes(bytes)
    }};
}

macro_rules! scalar_checks {
    ($module:ident, $ty:ty, $from_lsb:ident, $from_lt:ident, $select:ident) => {
        mod $module {
            use super::*;

            fn inputs() -> ($ty, $ty, $ty) {
                // Keep both RNG hooks linked: the BINSEC script uses both.
                black_box(PublicRng.next_u32());
                (
                    private_integer!($ty),
                    private_integer!($ty),
                    private_integer!($ty),
                )
            }

            #[checkct]
            pub fn choice_select() {
                let (a, b, condition) = inputs();
                black_box(select(a, b, condition));
            }

            #[inline(never)]
            fn select(a: $ty, b: $ty, condition: $ty) -> $ty {
                Choice::$from_lsb(condition).$select(a, b)
            }

            /// A single shift-and-subtract division step, as used in crypto-bigint.
            /// Keep the comparison and selection together to test their optimization.
            #[checkct]
            pub fn choice_select_after_lt() {
                let (a, b, _) = inputs();
                black_box(subtract_if_ge(a, b));
            }

            #[inline(never)]
            fn subtract_if_ge(a: $ty, b: $ty) -> $ty {
                let choice = Choice::$from_lt(a, b);
                choice.$select(a.wrapping_sub(b), a)
            }

            /// Compare the const helper with the cmov-backed trait on the same expression.
            #[checkct]
            pub fn ct_select_after_lt() {
                let (a, b, _) = inputs();
                black_box(subtract_if_ge_with_ct_select(a, b));
            }

            #[inline(never)]
            fn subtract_if_ge_with_ct_select(a: $ty, b: $ty) -> $ty {
                let choice = Choice::$from_lt(a, b);
                a.wrapping_sub(b).ct_select(&a, choice)
            }
        }
    };
}

scalar_checks!(u8_ops, u8, from_u8_lsb, from_u8_lt, select_u8);
scalar_checks!(u16_ops, u16, from_u16_lsb, from_u16_lt, select_u16);
scalar_checks!(u32_ops, u32, from_u32_lsb, from_u32_lt, select_u32);
scalar_checks!(u64_ops, u64, from_u64_lsb, from_u64_lt, select_u64);
// TODO: scalar_checks!(u128_ops, u128, from_u128_lsb, from_u128_lt, select_u128);
