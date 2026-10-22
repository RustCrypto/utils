//! Initial constant-time checks for the u32 and u64 primitives.
//!
//! Operands, conditions, and initial outputs are secret.
//! Keep the result live without asserting a predicate on a secret result.
use crate::rng::{PrivateRng, PublicRng};
use checkct_macros::checkct;
use cmov::{Cmov, CmovEq};
use core::hint::black_box;

#[inline]
fn private_byte() -> u8 {
    // The generated BINSEC script requires both RNG symbols, even though these
    // checks only need secret inputs. Keep the public hook linked as well.
    black_box(PublicRng.next_u32());
    let mut bytes = [0u8; 1];
    PrivateRng.fill_bytes(&mut bytes);
    bytes[0]
}

macro_rules! private_integer {
    ($ty:ty) => {{
        let mut bytes = [0u8; size_of::<$ty>()];
        PrivateRng.fill_bytes(&mut bytes);
        <$ty>::from_le_bytes(bytes)
    }};
}

macro_rules! scalar_checks {
    ($module:ident, $ty:ty) => {
        mod $module {
            use super::*;

            #[checkct]
            pub fn cmovnz() {
                let mut a = private_integer!($ty);
                let b = private_integer!($ty);
                a.cmovnz(&b, private_byte());
                black_box(a);
            }

            #[checkct]
            pub fn cmovz() {
                let mut a = private_integer!($ty);
                let b = private_integer!($ty);
                a.cmovz(&b, private_byte());
                black_box(a);
            }

            #[checkct]
            pub fn cmoveq() {
                let a = private_integer!($ty);
                let b = private_integer!($ty);
                let mut output = private_byte();
                a.cmoveq(&b, private_byte(), &mut output);
                black_box(output);
            }

            #[checkct]
            pub fn cmovne() {
                let a = private_integer!($ty);
                let b = private_integer!($ty);
                let mut output = private_byte();
                a.cmovne(&b, private_byte(), &mut output);
                black_box(output);
            }
        }
    };
}

scalar_checks!(u32_ops, u32);
scalar_checks!(u64_ops, u64);
