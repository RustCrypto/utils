//! Tests for `generic-array` interop for [`UInt`].
// TODO(tarcieri): phase this out as const generics become sufficiently powerful

use super::UIntEx;
use crypto_bigint::ArrayEncoding;
use hex_literal::hex;

/// Byte array that corresponds to `UIntEx`
type ByteArray = crypto_bigint::ByteArray<UIntEx>;

#[test]
#[cfg(target_pointer_width = "32")]
fn from_be_byte_array() {
    let n = UIntEx::from_be_byte_array(&hex!("0011223344556677").into());
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn from_be_byte_array() {
    let n = UIntEx::from_be_byte_array(&hex!("00112233445566778899aabbccddeeff").into());
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn from_le_byte_array() {
    let n = UIntEx::from_le_byte_array(&hex!("7766554433221100").into());
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn from_le_byte_array() {
    let n = UIntEx::from_le_byte_array(&hex!("ffeeddccbbaa99887766554433221100").into());
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn to_be_byte_array() {
    let expected_bytes = ByteArray::from(hex!("0011223344556677"));
    let actual_bytes = UIntEx::from_be_byte_array(&expected_bytes).to_be_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn to_be_byte_array() {
    let expected_bytes = ByteArray::from(hex!("00112233445566778899aabbccddeeff"));
    let actual_bytes = UIntEx::from_be_byte_array(&expected_bytes).to_be_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

#[test]
#[cfg(target_pointer_width = "32")]
fn to_le_byte_array() {
    let expected_bytes = ByteArray::from(hex!("7766554433221100"));
    let actual_bytes = UIntEx::from_le_byte_array(&expected_bytes).to_le_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn to_le_byte_array() {
    let expected_bytes = ByteArray::from(hex!("ffeeddccbbaa99887766554433221100"));
    let actual_bytes = UIntEx::from_le_byte_array(&expected_bytes).to_le_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}
