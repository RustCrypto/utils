//! Nonzero constructors with caller range information still visible.

use super::*;

macro_rules! nonzero_checks {
    ($module:ident, $ty:ty, $nz:ident) => {
        mod $module {
            use super::*;

            fn private() -> $ty {
                black_box(PublicRng.next_u32());
                private_integer!($ty)
            }

            #[checkct]
            pub fn unrestricted() {
                black_box(Choice::$nz(private()).to_u8());
            }

            #[checkct]
            pub fn shifted() {
                // The u64 case previously branched on Thumb-1.
                black_box(Choice::$nz(private() >> 1).to_u8());
            }

            #[checkct]
            pub fn sign_bit_clear() {
                black_box(Choice::$nz(private() & (<$ty>::MAX >> 1)).to_u8());
            }
        }
    };
}

nonzero_checks!(u8_ops, u8, from_u8_nz);
nonzero_checks!(u16_ops, u16, from_u16_nz);
nonzero_checks!(u32_ops, u32, from_u32_nz);
nonzero_checks!(u64_ops, u64, from_u64_nz);
nonzero_checks!(u128_ops, u128, from_u128_nz);

macro_rules! positive_nonzero_check {
    ($name:ident, $signed:ty, $unsigned:ty, $nz:ident) => {
        #[checkct]
        pub fn $name() {
            black_box(PublicRng.next_u32());
            let value = private_integer!($signed);
            black_box(
                Choice::$nz((value & !(value >> (<$signed>::BITS - 1))) as $unsigned).to_u8(),
            );
        }
    };
}

positive_nonzero_check!(positive_i8, i8, u8, from_u8_nz);
positive_nonzero_check!(positive_i16, i16, u16, from_u16_nz);
positive_nonzero_check!(positive_i32, i32, u32, from_u32_nz);

// On Thumb-1, the wide signed-mask argument itself can branch before ctutils
// receives it. Keep this composition regression on targets where the argument
// is constant-time; the shifted probes above exercise all widths on Thumb-1.
// Callers needing a signed comparison should bias the sign bit and use from_u*_lt.
#[cfg(not(all(target_arch = "arm", not(target_has_atomic = "ptr"))))]
positive_nonzero_check!(positive_i64, i64, u64, from_u64_nz);
#[cfg(not(all(target_arch = "arm", not(target_has_atomic = "ptr"))))]
positive_nonzero_check!(positive_i128, i128, u128, from_u128_nz);
