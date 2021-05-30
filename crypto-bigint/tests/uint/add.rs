//! Addition operation tests for [`UInt`].

use super::UIntEx;

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
