//! Traits which provide access to the optimized implementations of `cmov::{Cmov, CmovEq}` for
//! `[u8]` and `[u8; N]`.
//!
//! The trait impls for these types in the `cmov` crate coalesce word-sized chunks of bytes and then
//! use the underlying `Cmov`/`CmovEq` operation a word-at-a-time, for as many word-sized chunks
//! exist a given byte array/slice. The remainder are compared a byte-at-a-time in the event that
//! the array/slice length isn't a multiple of the word size.
//!
//! This is much more efficient than the generic impl which first converts each individual `u8` into
//! a word, then compares those, performing one CMOV-like operation per byte instead of per word.

use crate::Choice;
use cmov::{Cmov, CmovEq};

#[cfg(doc)]
use crate::{CtAssign, CtEq, CtSelect};

/// [`CtAssign`]-like trait impl'd for `[u8]` and `[u8; N]` providing optimized implementations
/// which perform better than the generic impl of [`CtAssign`] for `[T]` and `[T; N]`
/// where `T = u8`.
///
/// Ideally we would use [specialization] to provide more specific impls of these traits for these
/// types, but it's unstable and unlikely to be stabilized soon.
///
/// [specialization]: https://rust-lang.github.io/rfcs/1210-impl-specialization.html
pub trait BytesCtAssign: sealed::Sealed {
    /// Conditionally assign `other` to `self` if `choice` is [`Choice::TRUE`].
    fn bytes_ct_assign(&mut self, other: &Self, choice: Choice);
}

/// [`CtEq`]-like trait impl'd for `[u8]` and `[u8; N]` providing optimized implementations which
/// perform better than the generic impl of [`CtEq`] for `[T; N]` where `T = u8`.
///
/// Ideally we would use [specialization] to provide more specific impls of these traits for these
/// types, but it's unstable and unlikely to be stabilized soon.
///
/// [specialization]: https://rust-lang.github.io/rfcs/1210-impl-specialization.html
pub trait BytesCtEq<Rhs: ?Sized = Self>: sealed::Sealed {
    /// Determine if `self` is equal to the provided `value`.
    fn bytes_ct_eq(&self, other: &Rhs) -> Choice;

    /// Determine if `self` is NOT equal to the provided `value`.
    fn bytes_ct_ne(&self, other: &Rhs) -> Choice {
        !self.bytes_ct_eq(other)
    }
}

/// [`CtSelect`]-like trait impl'd for `[u8]` and `[u8; N]` providing optimized implementations
/// which perform better than the generic impl of [`CtSelect`] for `[T; N]` where `T = u8`.
///
/// Ideally we would use [specialization] to provide more specific impls of these traits for these
/// types, but it's unstable and unlikely to be stabilized soon.
///
/// [specialization]: https://rust-lang.github.io/rfcs/1210-impl-specialization.html
pub trait BytesCtSelect: BytesCtAssign + Sized {
    /// Select between `self` and `other` based on `choice`, returning a copy of the value.
    ///
    /// # Returns
    /// - `self` if `choice` is [`Choice::FALSE`].
    /// - `other` if `choice` is [`Choice::TRUE`].
    fn bytes_ct_select(&self, other: &Self, choice: Choice) -> Self;
}

impl BytesCtAssign for [u8] {
    #[inline]
    #[track_caller]
    fn bytes_ct_assign(&mut self, other: &Self, choice: Choice) {
        assert_eq!(
            self.len(),
            other.len(),
            "source slice length ({}) does not match destination slice length ({})",
            other.len(),
            self.len()
        );

        self.cmovnz(other, choice.into());
    }
}

impl BytesCtEq for [u8] {
    #[inline]
    fn bytes_ct_eq(&self, other: &[u8]) -> Choice {
        let mut ret = Choice::FALSE;
        self.cmoveq(other, 1, &mut ret.0);
        ret
    }
}

impl<const N: usize> BytesCtEq for [u8; N] {
    #[inline]
    fn bytes_ct_eq(&self, other: &[u8; N]) -> Choice {
        self.bytes_ct_eq(other.as_slice())
    }
}

impl<const N: usize> BytesCtAssign for [u8; N] {
    #[inline]
    fn bytes_ct_assign(&mut self, other: &Self, choice: Choice) {
        self.cmovnz(other, choice.into());
    }
}

impl<const N: usize> BytesCtEq<[u8]> for [u8; N] {
    #[inline]
    fn bytes_ct_eq(&self, other: &[u8]) -> Choice {
        let mut ret = Choice::FALSE;
        self.as_slice().cmoveq(other, 1, &mut ret.0);
        ret
    }
}

impl<const N: usize> BytesCtSelect for [u8; N] {
    #[inline]
    fn bytes_ct_select(&self, other: &Self, choice: Choice) -> Self {
        let mut ret = *self;
        ret.cmovnz(other, choice.into());
        ret
    }
}

mod sealed {
    /// Sealed trait to prevent others from adding impls of [`BytesExt`]. Instead, impls of the
    /// `Ct*` traits should be used.
    pub trait Sealed {}
    impl Sealed for [u8] {}
    impl<const N: usize> Sealed for [u8; N] {}
}

#[cfg(test)]
mod tests {
    use super::{BytesCtAssign, BytesCtEq, BytesCtSelect, Choice};

    mod array {
        use super::*;

        const EXAMPLE_A: [u8; 3] = [1, 2, 3];
        const EXAMPLE_B: [u8; 3] = [2, 2, 3];

        #[test]
        fn bytes_ct_eq() {
            assert!(EXAMPLE_A.bytes_ct_eq(&EXAMPLE_A).to_bool());
            assert!(!EXAMPLE_A.bytes_ct_eq(&EXAMPLE_B).to_bool());
        }

        #[test]
        fn bytes_ct_ne() {
            assert!(!EXAMPLE_A.bytes_ct_ne(&EXAMPLE_A).to_bool());
            assert!(EXAMPLE_A.bytes_ct_ne(&EXAMPLE_B).to_bool());
        }

        #[test]
        fn bytes_ct_select() {
            let should_be_a = EXAMPLE_A.bytes_ct_select(&EXAMPLE_B, Choice::FALSE);
            assert_eq!(EXAMPLE_A, should_be_a);

            let should_be_b = EXAMPLE_A.bytes_ct_select(&EXAMPLE_B, Choice::TRUE);
            assert_eq!(EXAMPLE_B, should_be_b);
        }
    }

    mod slice {
        use super::*;

        const EXAMPLE_A: &[u8] = &[1, 2, 3];
        const EXAMPLE_B: &[u8] = &[2, 2, 3];
        const EXAMPLE_C: &[u8] = &[1, 2];

        #[test]
        fn bytes_ct_assign() {
            let mut bytes = [0u8; 3];
            let slice = bytes.as_mut();

            slice.bytes_ct_assign(EXAMPLE_A, Choice::FALSE);
            assert_eq!(slice, &[0u8; 3]);
            slice.bytes_ct_assign(EXAMPLE_A, Choice::TRUE);
            assert_eq!(slice, EXAMPLE_A);
        }

        #[test]
        fn bytes_ct_eq() {
            assert!(EXAMPLE_A.bytes_ct_eq(EXAMPLE_A).to_bool());
            assert!(!EXAMPLE_A.bytes_ct_eq(EXAMPLE_B).to_bool());
            // different lengths
            assert!(!EXAMPLE_A.bytes_ct_eq(EXAMPLE_C).to_bool());
        }

        #[test]
        fn bytes_ct_ne() {
            assert!(!EXAMPLE_A.bytes_ct_ne(EXAMPLE_A).to_bool());
            assert!(EXAMPLE_A.bytes_ct_ne(EXAMPLE_B).to_bool());
            // different lengths
            assert!(EXAMPLE_A.bytes_ct_ne(EXAMPLE_C).to_bool());
        }
    }
}
