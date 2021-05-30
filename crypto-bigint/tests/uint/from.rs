//! Tests for various `From`-like conversions for `UInt`.

use super::UIntEx;
use crypto_bigint::U128;
use hex_literal::hex;

#[test]
fn from_u8() {
    let n = UIntEx::from(42u8);
    assert_eq!(n.limbs(), &[42, 0]);
}

#[test]
fn from_u16() {
    let n = UIntEx::from(42u16);
    assert_eq!(n.limbs(), &[42, 0]);
}

#[test]
fn from_u64() {
    let n = UIntEx::from(42u64);
    assert_eq!(n.limbs(), &[42, 0]);
}

#[test]
fn from_u128() {
    let n = U128::from(42u128);
    assert_eq!(&n.limbs()[..2], &[42, 0]);
    assert_eq!(u128::from(n), 42u128);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn from_be_bytes() {
    let bytes = hex!("0011223344556677");
    let n = UIntEx::from_be_bytes(&bytes);
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn from_be_bytes() {
    let bytes = hex!("00112233445566778899aabbccddeeff");
    let n = UIntEx::from_be_bytes(&bytes);
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn from_le_bytes() {
    let bytes = hex!("7766554433221100");
    let n = UIntEx::from_le_bytes(&bytes);
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn from_le_bytes() {
    let bytes = hex!("ffeeddccbbaa99887766554433221100");
    let n = UIntEx::from_le_bytes(&bytes);
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}
