//! Comparison operations.

use super::*;
use ctutils::{CtGt, CtLt};

macro_rules! comparison_checks {
    ($module:ident, $ty:ty) => {
        mod $module {
            use super::*;

            fn inputs() -> ($ty, $ty) {
                // Keep both RNG hooks linked: the BINSEC script uses both.
                black_box(PublicRng.next_u32());
                (private_integer!($ty), private_integer!($ty))
            }

            #[checkct]
            pub fn ct_lt() {
                let (a, b) = inputs();
                black_box(a.ct_lt(&b).to_u8());
            }

            #[checkct]
            pub fn ct_gt() {
                let (a, b) = inputs();
                black_box(a.ct_gt(&b).to_u8());
            }

            #[checkct]
            pub fn ct_select_after_lt() {
                let (a, b) = inputs();
                black_box(b.ct_select(&a, a.ct_lt(&b)));
            }

            #[checkct]
            pub fn ct_select_after_gt() {
                let (a, b) = inputs();
                black_box(b.ct_select(&a, a.ct_gt(&b)));
            }

            #[checkct]
            pub fn ct_select_independent_after_lt() {
                let (a, b) = inputs();
                let (x, y) = inputs();
                black_box(x.ct_select(&y, a.ct_lt(&b)));
            }

            #[checkct]
            pub fn ct_select_independent_after_gt() {
                let (a, b) = inputs();
                let (x, y) = inputs();
                black_box(x.ct_select(&y, a.ct_gt(&b)));
            }

            #[checkct]
            pub fn ct_select_after_lt_constant() {
                let (value, _) = inputs();
                let value = value & 63;
                black_box(value.ct_select(&2, (2 as $ty).ct_lt(&value)));
            }

            #[checkct]
            pub fn ct_select_after_gt_constant() {
                let (value, _) = inputs();
                let value = value & 63;
                black_box(value.ct_select(&2, (2 as $ty).ct_gt(&value)));
            }
        }
    };
}

comparison_checks!(u8_ops, u8);
comparison_checks!(u16_ops, u16);
comparison_checks!(u32_ops, u32);
comparison_checks!(u64_ops, u64);
comparison_checks!(u128_ops, u128);
comparison_checks!(usize_ops, usize);

macro_rules! constructor_checks {
    ($module:ident, $ty:ty, $from_lt:ident) => {
        mod $module {
            use super::*;

            fn private() -> $ty {
                black_box(PublicRng.next_u32());
                private_integer!($ty)
            }

            #[checkct]
            pub fn constant_lhs() {
                black_box(Choice::$from_lt(2, private() & 63).to_u8());
            }

            #[checkct]
            pub fn constant_rhs() {
                black_box(Choice::$from_lt(private() & 63, 2).to_u8());
            }

            #[checkct]
            pub fn select_constant() {
                let value = private() & 63;
                black_box(value.ct_select(&2, Choice::$from_lt(2, value)));
            }
        }
    };
}

constructor_checks!(u8_choice, u8, from_u8_lt);
constructor_checks!(u16_choice, u16, from_u16_lt);
constructor_checks!(u32_choice, u32, from_u32_lt);
constructor_checks!(u64_choice, u64, from_u64_lt);
constructor_checks!(u128_choice, u128, from_u128_lt);
