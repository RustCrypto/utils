//! Comparison specialization and loop-carried predicates used by crypto-bigint.

use super::*;

macro_rules! predicate_checks {
    ($module:ident, $ty:ty, $eq:ident, $le:ident, $lt:ident, $nz:ident, $select:ident,
        $leading:ident, $trailing:ident) => {
        mod $module {
            use super::*;

            // Inline these operations into the driver: an artificial function boundary
            // would hide the caller specialization which previously introduced branches.
            fn private() -> $ty {
                black_box(PublicRng.next_u32());
                private_integer!($ty)
            }

            #[checkct]
            pub fn lt_constant() {
                black_box(Choice::$lt(2, private() & 63).to_u8());
            }

            #[checkct]
            pub fn le_constant() {
                black_box(Choice::$le(2, private() & 63).to_u8());
            }

            #[checkct]
            pub fn eq_constant() {
                black_box(Choice::$eq(1, private() & 63).to_u8());
            }

            #[checkct]
            pub fn nz() {
                black_box(Choice::$nz(private()).to_u8());
            }

            #[checkct]
            pub fn min_constant() {
                let value = private() & 63;
                black_box(Choice::$lt(2, value).$select(value, 2));
            }

            #[checkct]
            pub fn and_loop() {
                let mut all_zero = Choice::TRUE;
                for value in [private(), private(), private()] {
                    // A raw byte predicate also covers callers that do not use a
                    // comparison constructor, such as crypto-bigint's Limb helpers.
                    let bit = (value | value.wrapping_neg()) >> (<$ty>::BITS - 1);
                    let zero = Choice::from_u8_lsb(bit as u8).not();
                    all_zero = all_zero.and(zero);
                }
                black_box(all_zero.to_u8());
            }

            fn limbs() -> [$ty; 16] {
                let mut limbs = [0; 16];
                for limb in &mut limbs {
                    *limb = private();
                }
                limbs
            }

            #[checkct]
            pub fn leading_zeros_choice_loop() {
                black_box(leading_zeros(&limbs()));
            }

            // Keep the whole limb scan intact, as in a crypto-bigint method. Each
            // scalar zero count remains inlined into the loop.
            #[inline(never)]
            fn leading_zeros(limbs: &[$ty; 16]) -> u32 {
                let mut count = 0;
                let mut all_zero = Choice::TRUE;
                let mut i = limbs.len();
                while i > 0 {
                    i -= 1;
                    let value = limbs[i];
                    count += all_zero.select_u32(0, ctutils::primitives::$leading(value));
                    // Match callers with their own scalar predicate helpers: do not
                    // rely on the comparison constructor's separate hardening here.
                    let bit = (value | value.wrapping_neg()) >> (<$ty>::BITS - 1);
                    all_zero = all_zero.and(Choice::from_u8_lsb(bit as u8).not());
                }
                count
            }

            #[checkct]
            pub fn trailing_zeros_choice_loop() {
                black_box(trailing_zeros(&limbs()));
            }

            #[inline(never)]
            fn trailing_zeros(limbs: &[$ty; 16]) -> u32 {
                let mut count = 0;
                let mut all_zero = Choice::TRUE;
                let mut i = 0;
                while i < limbs.len() {
                    let value = limbs[i];
                    count += all_zero.select_u32(0, ctutils::primitives::$trailing(value));
                    // Match callers with their own scalar predicate helpers: do not
                    // rely on the comparison constructor's separate hardening here.
                    let bit = (value | value.wrapping_neg()) >> (<$ty>::BITS - 1);
                    all_zero = all_zero.and(Choice::from_u8_lsb(bit as u8).not());
                    i += 1;
                }
                count
            }
        }
    };
}

predicate_checks!(
    u32_ops,
    u32,
    from_u32_eq,
    from_u32_le,
    from_u32_lt,
    from_u32_nz,
    select_u32,
    leading_zeros_u32,
    trailing_zeros_u32
);
predicate_checks!(
    u64_ops,
    u64,
    from_u64_eq,
    from_u64_le,
    from_u64_lt,
    from_u64_nz,
    select_u64,
    leading_zeros_u64,
    trailing_zeros_u64
);
