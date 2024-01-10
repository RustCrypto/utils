use hybrid_array::{Array, ArrayN};
use typenum::{U0, U2, U3, U4, U5, U6, U7};

const EXAMPLE_SLICE: &[u8] = &[1, 2, 3, 4, 5, 6];

/// Ensure `ArrayN` works as expected.
const _FOO: ArrayN<u8, 4> = Array([1, 2, 3, 4]);

#[test]
fn clone_from_slice() {
    let array = Array::<u8, U6>::clone_from_slice(EXAMPLE_SLICE);
    assert_eq!(array.as_slice(), EXAMPLE_SLICE);
}

#[test]
fn tryfrom_slice_for_array() {
    assert!(Array::<u8, U0>::try_from(EXAMPLE_SLICE).is_err());
    assert!(Array::<u8, U3>::try_from(EXAMPLE_SLICE).is_err());

    let array_ref = Array::<u8, U6>::try_from(EXAMPLE_SLICE).expect("slice contains 6 bytes");
    assert_eq!(&*array_ref, EXAMPLE_SLICE);

    assert!(Array::<u8, U7>::try_from(EXAMPLE_SLICE).is_err());
}

#[test]
fn tryfrom_slice_for_array_ref() {
    assert!(<&Array<u8, U0>>::try_from(EXAMPLE_SLICE).is_err());
    assert!(<&Array::<u8, U3>>::try_from(EXAMPLE_SLICE).is_err());

    let array_ref = <&Array<u8, U6>>::try_from(EXAMPLE_SLICE).expect("slice contains 6 bytes");
    assert_eq!(array_ref.as_slice(), EXAMPLE_SLICE);

    assert!(<&Array::<u8, U7>>::try_from(EXAMPLE_SLICE).is_err());
}

#[test]
fn concat() {
    let prefix = Array::<u8, U2>::clone_from_slice(&EXAMPLE_SLICE[..2]);
    let suffix = Array::<u8, U4>::clone_from_slice(&EXAMPLE_SLICE[2..]);

    let array = prefix.concat(suffix);
    assert_eq!(array.as_slice(), EXAMPLE_SLICE);
}

#[test]
fn split() {
    let array = Array::<u8, U6>::clone_from_slice(EXAMPLE_SLICE);

    let (prefix, suffix) = array.split::<U2>();

    assert_eq!(prefix.as_slice(), &EXAMPLE_SLICE[..2]);
    assert_eq!(suffix.as_slice(), &EXAMPLE_SLICE[2..]);
}

#[test]
fn split_ref() {
    let array = Array::<u8, U6>::clone_from_slice(EXAMPLE_SLICE);

    let (prefix, suffix) = array.split_ref::<U3>();

    assert_eq!(prefix.as_slice(), &EXAMPLE_SLICE[..3]);
    assert_eq!(suffix.as_slice(), &EXAMPLE_SLICE[3..]);
}

#[test]
fn split_ref_mut() {
    let array = &mut Array::<u8, U6>::clone_from_slice(EXAMPLE_SLICE);

    let (prefix, suffix) = array.split_ref_mut::<U4>();

    assert_eq!(prefix.as_slice(), &EXAMPLE_SLICE[..4]);
    assert_eq!(suffix.as_slice(), &EXAMPLE_SLICE[4..]);
}

#[test]
fn from_iterator_correct_size() {
    let array: Array<u8, U6> = EXAMPLE_SLICE.iter().copied().collect();
    assert_eq!(array.as_slice(), EXAMPLE_SLICE);
}

#[test]
#[should_panic]
fn from_iterator_too_short() {
    let _array: Array<u8, U7> = EXAMPLE_SLICE.iter().copied().collect();
}

#[test]
#[should_panic]
fn from_iterator_too_long() {
    let _array: Array<u8, U5> = EXAMPLE_SLICE.iter().copied().collect();
}
