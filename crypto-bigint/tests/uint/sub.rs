//! Subtraction operation tests for [`UInt`].

use super::UIntEx;
use crypto_bigint::Limb;

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
