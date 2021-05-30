//! Hexadecimal (de)serialization support for [`UInt`].

use super::UIntEx;
use crypto_bigint::U128;

#[test]
#[cfg(target_pointer_width = "32")]
fn from_be_hex() {
    let n = UIntEx::from_be_hex("0011223344556677");
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn from_be_hex() {
    let n = UIntEx::from_be_hex("00112233445566778899aabbccddeeff");
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn from_le_hex() {
    let n = UIntEx::from_le_hex("7766554433221100");
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn from_le_hex() {
    let n = U128::from_le_hex("ffeeddccbbaa99887766554433221100");
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
fn hex_upper() {
    let hex = "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD";
    let n = U128::from_be_hex(hex);
    assert_eq!(hex, format!("{:X}", n));
}

#[test]
fn hex_lower() {
    let hex = "aaaaaaaabbbbbbbbccccccccdddddddd";
    let n = U128::from_be_hex(hex);
    assert_eq!(hex, format!("{:x}", n));
}
