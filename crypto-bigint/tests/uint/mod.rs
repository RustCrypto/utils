//! [`UInt`] integration tests.

mod add;
mod from;
mod hex;
mod mul;
mod sub;

#[cfg(feature = "generic-array")]
mod array;

use crypto_bigint::{Concat, Split, U128, U64};

// 2-limb example that's twice as wide as the native word size
#[cfg(target_pointer_width = "64")]
use crypto_bigint::U128 as UIntEx;
#[cfg(target_pointer_width = "32")]
use crypto_bigint::U64 as UIntEx;

#[test]
fn display() {
    let hex = "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD";
    let n = U128::from_be_hex(hex);
    assert_eq!(hex, n.to_string());
}

#[test]
fn is_zero() {
    assert!(bool::from(UIntEx::ZERO.is_zero()));
    assert!(!bool::from(UIntEx::ONE.is_zero()));
}

#[test]
fn square() {
    let n = U64::from_u64(0xffff_ffff_ffff_ffff);
    let (hi, lo) = n.square().split();
    assert_eq!(lo, U64::from_u64(1));
    assert_eq!(hi, U64::from_u64(0xffff_ffff_ffff_fffe));
}

#[test]
fn concat() {
    let hi = U64::from_u64(0x0011223344556677);
    let lo = U64::from_u64(0x8899aabbccddeeff);
    assert_eq!(
        hi.concat(&lo),
        U128::from_be_hex("00112233445566778899aabbccddeeff")
    );
}

#[test]
fn split() {
    let (hi, lo) = U128::from_be_hex("00112233445566778899aabbccddeeff").split();
    assert_eq!(hi, U64::from_u64(0x0011223344556677));
    assert_eq!(lo, U64::from_u64(0x8899aabbccddeeff));
}
