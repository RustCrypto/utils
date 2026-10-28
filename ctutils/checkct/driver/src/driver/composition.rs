//! Scalar predicates composed with selection, containers, and wrappers.

use super::*;
use ctutils::{CtEq, CtFind, CtGt, CtLookup, CtLt, CtOption};

fn condition() -> Choice {
    Choice::from_u8_lsb(private_integer!(u8))
}

macro_rules! composition_checks {
    ($module:ident, $ty:ty) => {
        mod $module {
            use super::*;

            fn private() -> $ty {
                black_box(PublicRng.next_u32());
                private_integer!($ty)
            }

            fn array() -> [$ty; 4] {
                core::array::from_fn(|_| private())
            }

            fn option() -> CtOption<$ty> {
                CtOption::new(private(), condition())
            }

            #[checkct]
            pub fn equality_select() {
                let (a, b) = (private(), private());
                black_box(a.ct_select(&b, a.ct_eq(&b)));
                black_box(a.ct_ne(&b).to_u8());
            }

            #[checkct]
            pub fn array_select() {
                black_box(array().ct_select(&array(), condition()));
            }

            #[checkct]
            pub fn array_equality() {
                black_box(array().ct_eq(&array()).to_u8());
            }

            #[checkct]
            pub fn option_unwrap_or() {
                black_box(option().unwrap_or(private()));
            }

            #[checkct]
            pub fn option_or() {
                black_box(option().or(option()).unwrap_or(private()));
            }

            #[checkct]
            pub fn option_and_then() {
                let other = option();
                black_box(option().and_then(|_| other).unwrap_or(private()));
            }

            #[checkct]
            pub fn option_equality() {
                black_box(option().ct_eq(&option()).to_u8());
            }

            #[checkct]
            pub fn option_select() {
                let selected = option().ct_select(&option(), condition());
                black_box(selected.is_some().to_u8());
                black_box(selected.unwrap_or(private()));
            }

            #[checkct]
            pub fn lookup() {
                // An unrestricted secret index includes every hit and out-of-range miss.
                let index = private_integer!(usize);
                let found = array().ct_lookup(index);
                black_box(found.is_some().to_u8());
                black_box(found.unwrap_or(private()));
            }

            #[checkct]
            pub fn find() {
                // Includes no match, multiple matches, and first/last matches.
                let needle = private();
                let found = array().ct_find(|value| value.ct_eq(&needle));
                black_box(found.is_some().to_u8());
                black_box(found.unwrap_or(private()));
            }
        }
    };
}

composition_checks!(u32_ops, u32);
composition_checks!(u64_ops, u64);

macro_rules! nonzero_comparison_checks {
    ($module:ident, $ty:ident, $int:ty) => {
        mod $module {
            use super::*;

            fn private() -> core::num::$ty {
                black_box(PublicRng.next_u32());
                core::num::$ty::new(private_integer!($int) | 1).expect("low bit is set")
            }

            #[checkct]
            pub fn select_after_lt() {
                let (a, b) = (private(), private());
                black_box(b.ct_select(&a, a.ct_lt(&b)));
            }

            #[checkct]
            pub fn select_after_gt() {
                let (a, b) = (private(), private());
                black_box(b.ct_select(&a, a.ct_gt(&b)));
            }
        }
    };
}

nonzero_comparison_checks!(nonzero_u8, NonZeroU8, u8);
nonzero_comparison_checks!(nonzero_u16, NonZeroU16, u16);
nonzero_comparison_checks!(nonzero_u32, NonZeroU32, u32);
nonzero_comparison_checks!(nonzero_u64, NonZeroU64, u64);
nonzero_comparison_checks!(nonzero_u128, NonZeroU128, u128);
nonzero_comparison_checks!(nonzero_usize, NonZeroUsize, usize);
