//! Negation operations.

// Run with and without `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true`. Secret inputs include MIN.
// A public false condition must not trigger a secret-dependent overflow check while eagerly
// computing the other operand.

use super::*;
use ctutils::CtNeg;

macro_rules! negation_checks {
    ($module:ident, $ty:ty, $input:expr) => {
        mod $module {
            use super::*;

            fn inputs() -> ($ty, Choice) {
                black_box(PublicRng.next_u32());
                ($input, Choice::from_u8_lsb(private_integer!(u8)))
            }

            #[checkct]
            pub fn ct_neg() {
                let (value, choice) = inputs();
                black_box(value.ct_neg(choice));
            }

            #[checkct]
            pub fn ct_neg_assign() {
                let (mut value, choice) = inputs();
                value.ct_neg_assign(choice);
                black_box(value);
            }

            #[checkct]
            pub fn ct_neg_false() {
                let (value, _) = inputs();
                black_box(value.ct_neg(Choice::FALSE));
            }

            #[checkct]
            pub fn ct_neg_assign_false() {
                let (mut value, _) = inputs();
                value.ct_neg_assign(Choice::FALSE);
                black_box(value);
            }
        }
    };
}

negation_checks!(i8_ops, i8, private_integer!(i8));
negation_checks!(i16_ops, i16, private_integer!(i16));
negation_checks!(i32_ops, i32, private_integer!(i32));
negation_checks!(i64_ops, i64, private_integer!(i64));
negation_checks!(i128_ops, i128, private_integer!(i128));
negation_checks!(isize_ops, isize, private_integer!(isize));

// Setting the sign bit guarantees NonZero's invariant without excluding MIN or introducing a
// secret-dependent branch in the harness.
macro_rules! nonzero_checks {
    ($module:ident, $ty:ident, $int:ty) => {
        negation_checks!(
            $module,
            core::num::$ty,
            core::num::$ty::new(private_integer!($int) | <$int>::MIN).expect("sign bit is set")
        );
    };
}

nonzero_checks!(nonzero_i8, NonZeroI8, i8);
nonzero_checks!(nonzero_i16, NonZeroI16, i16);
nonzero_checks!(nonzero_i32, NonZeroI32, i32);
nonzero_checks!(nonzero_i64, NonZeroI64, i64);
nonzero_checks!(nonzero_i128, NonZeroI128, i128);
nonzero_checks!(nonzero_isize, NonZeroIsize, isize);
