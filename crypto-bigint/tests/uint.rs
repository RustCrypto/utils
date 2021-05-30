//! [`UInt`] integration tests.
// TODO(tarcieri): refactor these tests into separate submodules per operation

use crypto_bigint::{Concat, Limb, Split, U128, U64};
use hex_literal::hex;

// 2-limb example that's twice as wide as the native word size
#[cfg(target_pointer_width = "64")]
use crypto_bigint::U128 as UIntEx;
#[cfg(target_pointer_width = "32")]
use crypto_bigint::U64 as UIntEx;

/// Byte array that corresponds to `UIntEx`
#[cfg(feature = "generic-array")]
use crypto_bigint::ArrayEncoding;
#[cfg(feature = "generic-array")]
type ByteArray = crypto_bigint::ByteArray<UIntEx>;

#[test]
fn from_u8() {
    let n = UIntEx::from_u8(42u8);
    assert_eq!(n.limbs(), &[42, 0]);
}

#[test]
fn from_u16() {
    let n = UIntEx::from_u16(42u16);
    assert_eq!(n.limbs(), &[42, 0]);
}

#[test]
fn from_u64() {
    let n = UIntEx::from_u64(42u64);
    assert_eq!(n.limbs(), &[42, 0]);
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

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "32")]
fn from_be_byte_array() {
    let n = UIntEx::from_be_byte_array(&hex!("0011223344556677").into());
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "64")]
fn from_be_byte_array() {
    let n = UIntEx::from_be_byte_array(&hex!("00112233445566778899aabbccddeeff").into());
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "32")]
fn from_le_byte_array() {
    let n = UIntEx::from_le_byte_array(&hex!("7766554433221100").into());
    assert_eq!(n.limbs(), &[0x44556677, 0x00112233]);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "64")]
fn from_le_byte_array() {
    let n = UIntEx::from_le_byte_array(&hex!("ffeeddccbbaa99887766554433221100").into());
    assert_eq!(n.limbs(), &[0x8899aabbccddeeff, 0x0011223344556677]);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "32")]
fn to_be_byte_array() {
    let expected_bytes = ByteArray::from(hex!("0011223344556677"));
    let actual_bytes = UIntEx::from_be_byte_array(&expected_bytes).to_be_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "64")]
fn to_be_byte_array() {
    let expected_bytes = ByteArray::from(hex!("00112233445566778899aabbccddeeff"));
    let actual_bytes = UIntEx::from_be_byte_array(&expected_bytes).to_be_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "32")]
fn to_le_byte_array() {
    let expected_bytes = ByteArray::from(hex!("7766554433221100"));
    let actual_bytes = UIntEx::from_le_byte_array(&expected_bytes).to_le_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

#[test]
#[cfg(feature = "generic-array")]
#[cfg(target_pointer_width = "64")]
fn to_le_byte_array() {
    let expected_bytes = ByteArray::from(hex!("ffeeddccbbaa99887766554433221100"));
    let actual_bytes = UIntEx::from_le_byte_array(&expected_bytes).to_le_byte_array();
    assert_eq!(expected_bytes, actual_bytes);
}

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
fn adc_no_carry() {
    let (res, carry) = UIntEx::ZERO.adc(&UIntEx::ONE, 0);
    assert_eq!(res, UIntEx::ONE);
    assert_eq!(carry, 0);
}

#[test]
fn adc_with_carry() {
    let (res, carry) = UIntEx::MAX.adc(&UIntEx::ONE, 0);

    assert_eq!(res, UIntEx::ZERO);
    assert_eq!(carry, 1);
}

#[test]
fn sbb_no_borrow() {
    let (res, borrow) = UIntEx::ONE.sbb(&UIntEx::ONE, 0);
    assert_eq!(res, UIntEx::ZERO);
    assert_eq!(borrow, 0);
}

#[test]
fn sbb_with_borrow() {
    let (res, borrow) = UIntEx::ZERO.sbb(&UIntEx::ONE, 0);

    assert_eq!(res, UIntEx::MAX);
    assert_eq!(borrow, Limb::MAX);
}

#[test]
fn wrapping_add_no_carry() {
    assert_eq!(UIntEx::ZERO.wrapping_add(&UIntEx::ONE), UIntEx::ONE);
}

#[test]
fn wrapping_add_with_carry() {
    assert_eq!(UIntEx::MAX.wrapping_add(&UIntEx::ONE), UIntEx::ZERO);
}

#[test]
fn checked_add_ok() {
    let result = UIntEx::ZERO.checked_add(&UIntEx::ONE);
    assert_eq!(result.unwrap(), UIntEx::ONE);
}

#[test]
fn checked_add_overflow() {
    let result = UIntEx::MAX.checked_add(&UIntEx::ONE);
    assert!(!bool::from(result.is_some()));
}

#[test]
fn wrapping_sub_no_borrow() {
    assert_eq!(UIntEx::ONE.wrapping_sub(&UIntEx::ONE), UIntEx::ZERO);
}

#[test]
fn wrapping_sub_with_borrow() {
    assert_eq!(UIntEx::ZERO.wrapping_sub(&UIntEx::ONE), UIntEx::MAX);
}

#[test]
fn checked_sub_ok() {
    let result = UIntEx::ONE.checked_sub(&UIntEx::ONE);
    assert_eq!(result.unwrap(), UIntEx::ZERO);
}

#[test]
fn checked_sub_overflow() {
    let result = UIntEx::ZERO.checked_sub(&UIntEx::ONE);
    assert!(!bool::from(result.is_some()));
}

#[test]
fn mul_wide_zero_and_one() {
    assert_eq!(
        UIntEx::ZERO.mul_wide(&UIntEx::ZERO),
        (UIntEx::ZERO, UIntEx::ZERO)
    );
    assert_eq!(
        UIntEx::ZERO.mul_wide(&UIntEx::ONE),
        (UIntEx::ZERO, UIntEx::ZERO)
    );
    assert_eq!(
        UIntEx::ONE.mul_wide(&UIntEx::ZERO),
        (UIntEx::ZERO, UIntEx::ZERO)
    );
    assert_eq!(
        UIntEx::ONE.mul_wide(&UIntEx::ONE),
        (UIntEx::ZERO, UIntEx::ONE)
    );
}

// TODO(tarcieri): add proptests for multiplication
#[test]
fn mul_wide_lo_only() {
    let primes: &[u32] = &[3, 5, 17, 256, 65537];

    for &a_int in primes {
        for &b_int in primes {
            let (hi, lo) = U64::from_u32(a_int).mul_wide(&U64::from_u32(b_int));
            let expected = U64::from_u64(a_int as u64 * b_int as u64);
            assert_eq!(lo, expected);
            assert!(bool::from(hi.is_zero()));
        }
    }
}

#[test]
fn square() {
    let n = U64::from_u64(0xffff_ffff_ffff_ffff);
    let (hi, lo) = n.square().split();
    assert_eq!(lo, U64::from_u64(1));
    assert_eq!(hi, U64::from_u64(0xffff_ffff_ffff_fffe));
}

#[test]
fn checked_mul_ok() {
    let n = U64::from_u32(0xffff_ffff);
    assert_eq!(
        n.checked_mul(&n).unwrap(),
        U64::from_u64(0xffff_fffe_0000_0001)
    );
}

#[test]
fn checked_mul_overflow() {
    let n = U64::from_u64(0xffff_ffff_ffff_ffff);
    assert!(bool::from(n.checked_mul(&n).is_none()));
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
