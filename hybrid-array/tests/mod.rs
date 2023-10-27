use hybrid_array::{Array, ByteArray};
use typenum::{U0, U2, U3, U4, U6, U7};

const EXAMPLE_SLICE: &[u8] = &[1, 2, 3, 4, 5, 6];

#[test]
fn clone_from_slice() {
    let array = Array::<u8, U6>::clone_from_slice(EXAMPLE_SLICE);
    assert_eq!(array.as_slice(), EXAMPLE_SLICE);
}

#[test]
fn tryfrom_slice_for_array() {
    assert!(ByteArray::<U0>::try_from(EXAMPLE_SLICE).is_err());
    assert!(ByteArray::<U3>::try_from(EXAMPLE_SLICE).is_err());

    let array_ref = ByteArray::<U6>::try_from(EXAMPLE_SLICE).expect("slice contains 6 bytes");
    assert_eq!(&*array_ref, EXAMPLE_SLICE);

    assert!(ByteArray::<U7>::try_from(EXAMPLE_SLICE).is_err());
}

#[test]
fn tryfrom_slice_for_array_ref() {
    assert!(<&ByteArray<U0>>::try_from(EXAMPLE_SLICE).is_err());
    assert!(<&ByteArray::<U3>>::try_from(EXAMPLE_SLICE).is_err());

    let array_ref = <&ByteArray<U6>>::try_from(EXAMPLE_SLICE).expect("slice contains 6 bytes");
    assert_eq!(array_ref.as_slice(), EXAMPLE_SLICE);

    assert!(<&ByteArray::<U7>>::try_from(EXAMPLE_SLICE).is_err());
}

#[test]
fn concat() {
    let prefix = ByteArray::<U2>::clone_from_slice(&EXAMPLE_SLICE[..2]);
    let suffix = ByteArray::<U4>::clone_from_slice(&EXAMPLE_SLICE[2..]);

    let array = prefix.concat(suffix);
    assert_eq!(array.as_slice(), EXAMPLE_SLICE);
}

#[test]
fn split() {
    let array = ByteArray::<U6>::clone_from_slice(EXAMPLE_SLICE);

    let (prefix, suffix) = array.split::<U2>();

    assert_eq!(prefix.as_slice(), &EXAMPLE_SLICE[..2]);
    assert_eq!(suffix.as_slice(), &EXAMPLE_SLICE[2..]);
}

#[test]
fn split_ref() {
    let array = ByteArray::<U6>::clone_from_slice(EXAMPLE_SLICE);

    let (prefix, suffix) = array.split_ref::<U3>();

    assert_eq!(prefix.as_slice(), &EXAMPLE_SLICE[..3]);
    assert_eq!(suffix.as_slice(), &EXAMPLE_SLICE[3..]);
}

#[test]
fn split_ref_mut() {
    let array = &mut ByteArray::<U6>::clone_from_slice(EXAMPLE_SLICE);

    let (prefix, suffix) = array.split_ref_mut::<U4>();

    assert_eq!(prefix.as_slice(), &EXAMPLE_SLICE[..4]);
    assert_eq!(suffix.as_slice(), &EXAMPLE_SLICE[4..]);
}
