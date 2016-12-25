use core;

use super::{add_bytes_to_bits, add_bytes_to_bits_tuple};

// A normal addition - no overflow occurs
#[test]
fn test_add_bytes_to_bits_ok() {
    assert!(add_bytes_to_bits(100, 10) == 180);
}

// A simple failure case - adding 1 to the max value
#[test]
#[should_panic]
fn test_add_bytes_to_bits_overflow() { add_bytes_to_bits(core::u64::MAX, 1); }

// A normal addition - no overflow occurs (fast path)
#[test]
fn test_add_bytes_to_bits_tuple_ok() {
    assert!(add_bytes_to_bits_tuple((5, 100), 10) == (5, 180));
}

// The low order value overflows into the high order value
#[test]
fn test_add_bytes_to_bits_tuple_ok2() {
    assert!(add_bytes_to_bits_tuple((5, core::u64::MAX), 1) == (6, 7));
}

// The value to add is too large to be converted into bits without overflowing
// its type
#[test]
fn test_add_bytes_to_bits_tuple_ok3() {
    assert!(add_bytes_to_bits_tuple((5, 0), 0x4000000000000001) == (7, 8));
}

// A simple failure case - adding 1 to the max value
#[test]
#[should_panic]
fn test_add_bytes_to_bits_tuple_overflow() {
    add_bytes_to_bits_tuple((core::u64::MAX, core::u64::MAX), 1);
}

// The value to add is too large to convert to bytes without overflowing its
// type, but the high order value from this conversion overflows when added to
// the existing high order value
#[test]
#[should_panic]
fn test_add_bytes_to_bits_tuple_overflow2() {
    let value: u64 = core::u64::MAX;
    add_bytes_to_bits_tuple((value - 1, 0), 0x8000000000000000);
}
