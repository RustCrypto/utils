//! Utilities for leveraging the optimized implementations of `Cmov`/`CmovEq` for types whose size
//! is 1-byte.

use crate::{Choice, traits::no_uninit::NoUninit};
use cmov::CmovEq;
use core::slice;

/// Perform constant-time equality comparison on slices of 1-byte sized types using the optimized
/// implementation of `CmovEq` for byte slices.
pub(crate) fn ct_eq<T: NoUninit>(a: &[T], b: &[T]) -> Choice {
    assert_eq!(
        size_of::<T>(),
        1,
        "this function is intended for 1-byte sized types"
    );

    // SAFETY:
    // - We asserted above that `size_of::<T>() == size_of::<u8>() == 1`.
    // - The `NoUninit` bound ensures the type does not contain uninitialized memory.
    // - We don't need to worry about alignment because all types are 1-byte.
    // - 1-byte is too small to contain a pointer/reference.
    // - We source the slice length directly from the other valid slice.
    #[allow(unsafe_code)]
    let (a, b) = unsafe {
        (
            slice::from_raw_parts(a.as_ptr() as *const u8, a.len()),
            slice::from_raw_parts(b.as_ptr() as *const u8, b.len()),
        )
    };

    let mut ret = Choice::FALSE;
    a.cmoveq(b, 1, &mut ret.0);
    ret
}

#[cfg(test)]
mod tests {
    use core::num::{NonZeroI8, NonZeroU8};

    macro_rules! ct_eq_test {
        ($name:ident, $a:expr, $b:expr) => {
            #[test]
            fn $name() {
                let x = $a;
                let y = $b;

                let a = [x, x, x];
                let b = [x, x, y];
                let c = [x, y, y];
                let d = [y, y, y];

                assert!(super::ct_eq(&a, &a).to_bool());
                assert!(super::ct_eq(&b, &b).to_bool());
                assert!(super::ct_eq(&c, &c).to_bool());
                assert!(super::ct_eq(&d, &d).to_bool());

                for rhs in &[b, c, d] {
                    assert!(!super::ct_eq(&a, rhs).to_bool());
                }
            }
        };
    }

    ct_eq_test!(i8_ct_eq, 1i8, 2i8);
    ct_eq_test!(u8_ct_eq, 1u8, 2u8);
    ct_eq_test!(
        non_zero_i8_ct_eq,
        NonZeroI8::new(1i8).unwrap(),
        NonZeroI8::new(2i8).unwrap()
    );
    ct_eq_test!(
        non_zero_u8_ct_eq,
        NonZeroU8::new(1u8).unwrap(),
        NonZeroU8::new(2u8).unwrap()
    );
}
