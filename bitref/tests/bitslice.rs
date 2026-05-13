//! `BitSlice` integration tests.

#![expect(clippy::unwrap_used, reason = "tests")]

use bitref::BitSlice;
use core::ops::Range;

const BYTES: [u8; 2] = [0xa0, 0x0a];
const BITS: [bool; 16] = [
    true, false, true, false, false, false, false, false, false, false, false, false, true, false,
    true, false,
];

#[test]
fn debug() {
    let bits = BitSlice::new(&BYTES);
    assert_eq!(&format!("{bits:?}"), "BitSlice([1010000000001010])");

    let bits2 = &bits[1..14];
    assert_eq!(&format!("{bits2:?}"), "BitSlice([0100000000010])");
}

#[test]
fn first() {
    assert!(BitSlice::EMPTY.first().is_none());
    assert!(!BitSlice::new(&[0]).first().unwrap());
}

#[test]
fn last() {
    assert!(BitSlice::EMPTY.last().is_none());
    assert!(BitSlice::new(&[0, 1]).last().unwrap());
}

#[test]
fn get_bit() {
    let bits = BitSlice::new(&BYTES);

    for (i, expected) in BITS.into_iter().enumerate() {
        assert_eq!(bits.get_bit(i).unwrap(), expected);
    }

    assert!(bits.get_bit(bits.len()).is_err());
}

#[test]
fn get_slice() {
    let bits = BitSlice::new(&BYTES);

    // Ensure `BitSlice::get_slice` behaves like `&[bool]`.
    for i in 0..BITS.len() {
        for j in i..BITS.len() {
            let bitslice = bits.get_slice(i..j).unwrap();
            verify_against_expected(bitslice, i..j);
        }
    }
}

// TODO(tarcieri): test mutations
#[test]
fn get_mut_slice() {
    let mut bytes = BYTES;
    let bits = BitSlice::new_mut(&mut bytes);

    // Ensure `BitSlice::get_mut_slice` behaves like `&mut [bool]`.
    for i in 0..BITS.len() {
        for j in i..BITS.len() {
            let bitslice = bits.get_mut_slice(i..j).unwrap();
            verify_against_expected(bitslice, i..j);
        }
    }
}

#[test]
fn index_range_slicing() {
    let bits = BitSlice::new(&BYTES);

    // Ensure `&bitslice[i..j]` behaves like `&[bool]`.
    for i in 0..BITS.len() {
        for j in i..BITS.len() {
            verify_against_expected(&bits[i..j], i..j);
        }
    }
}

// TODO(tarcieri): test `IndexMut<RangeFrom>`
#[test]
fn index_range_from_slicing() {
    let bits = BitSlice::new(&BYTES);

    // Ensure `&bitslice[i..]` behaves like `&[bool]`.
    for i in 0..BITS.len() {
        verify_against_expected(&bits[i..], i..BITS.len());
    }
}

// TODO(tarcieri): test mutations
#[test]
fn index_mut_range_slicing() {
    let mut bytes = BYTES;
    let bits = BitSlice::new_mut(&mut bytes);

    // Ensure `&mut bitslice[i..j]` behaves like `&mut [bool]`.
    for i in 0..BITS.len() {
        for j in i..BITS.len() {
            verify_against_expected(&bits[i..j], i..j);
        }
    }
}

#[test]
fn is_empty() {
    assert!(BitSlice::EMPTY.is_empty());
    assert!(!BitSlice::new(&[0]).is_empty());
    assert!(!BitSlice::new(&BYTES).is_empty());
}

#[test]
fn len() {
    assert_eq!(BitSlice::EMPTY.len(), 0);
    assert_eq!(BitSlice::new(&[0]).len(), 8);
    assert_eq!(BitSlice::new(&BYTES).len(), 16);
}

#[test]
fn set_bit() {
    let mut bytes = [0x0, 0x0];
    let bits = BitSlice::new_mut(&mut bytes);

    for (i, bit) in BITS.into_iter().enumerate() {
        bits.set_bit(i, bit).unwrap();
    }

    assert!(bits.set_bit(bits.len(), true).is_err());
    assert_eq!(bytes, BYTES);
}

#[test]
fn split_first() {
    assert!(BitSlice::EMPTY.split_first().is_none());
    let bits = BitSlice::new(&[0b10000000]);
    let (first, rest) = bits.split_first().unwrap();
    assert!(first);
    assert_eq!(rest, &BitSlice::new(&[0])[..7]);
}

#[test]
fn split_last() {
    assert!(BitSlice::EMPTY.split_last().is_none());
    let bits = BitSlice::new(&[1]);
    let (last, rest) = bits.split_last().unwrap();
    assert!(last);
    assert_eq!(rest, &BitSlice::new(&[0])[..7]);
}

#[test]
fn replace_bit() {
    let mut bytes = [!BYTES[0], !BYTES[1]];
    let bits = BitSlice::new_mut(&mut bytes);

    for (i, bit) in BITS.into_iter().enumerate() {
        let old = bits.replace_bit(i, bit).unwrap();
        assert_eq!(old, !bit);
    }

    assert_eq!(bytes, BYTES);
}

/// Check `bitslice` matches the given `range` of `BITS`.
#[track_caller]
fn verify_against_expected(bitslice: &BitSlice, range: Range<usize>) {
    let coreslice = &BITS[range];
    assert_eq!(bitslice.len(), coreslice.len());

    for k in 0..bitslice.len() {
        assert_eq!(bitslice.get_bit(k).unwrap(), coreslice[k]);
    }
}
