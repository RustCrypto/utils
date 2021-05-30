//! Multiplication operation tests for [`UInt`].

use super::UIntEx;
use crypto_bigint::U64;

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
