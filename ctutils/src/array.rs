//! Array helpers: generic selection/equality operations for arrays.
//!
//! Use these in the event there isn't an impl of a given trait for `[T; N]`. If there is, however,
//! you should prefer that for performance reasons.

use crate::{Choice, CtAssign, CtEq, CtSelect, slice};

/// Generic implementation of conditional assignment for arrays which works with any type which
/// impls `CtSelect`. Useful in the event there isn't a `CtAssign` impl for `[T; N]`.
///
/// Assigns `src` to `dst` in constant-time when `choice` is [`Choice::TRUE`].
///
/// Unfortunately we can't provide this as a trait impl without specialization, since it would
/// overlap with the optimized type-specific impls we provide.
#[inline]
pub fn ct_assign<T, const N: usize>(dst: &mut [T; N], src: &[T; N], choice: Choice)
where
    T: CtAssign,
{
    slice::ct_assign(dst, src, choice);
}

/// Generic implementation of constant-time equality testing for arrays which works with any type
/// which impls `CtEq`. Useful in the event there isn't a `CtEq` impl for `[T; N]`.
#[inline]
pub fn ct_eq<T, const N: usize>(a: &[T; N], b: &[T; N]) -> Choice
where
    T: CtEq,
{
    let mut ret = Choice::TRUE;
    for (a, b) in a.iter().zip(b.iter()) {
        ret &= a.ct_eq(b);
    }
    ret
}

/// Generic implementation of conditional selection for arrays which works with any type which
/// impls `CtSelect`. Useful in the event there isn't a `CtSelect` impl for `[T; N]`.
///
/// Selects `a` if `choice` is `Choice::FALSE`, and `b` if `choice` is `Choice::TRUE`.
///
/// Unfortunately we can't provide this as a trait impl without specialization, since it would
/// overlap with the optimized type-specific impls we provide.
#[inline]
pub fn ct_select<T, const N: usize>(a: &[T; N], b: &[T; N], choice: Choice) -> [T; N]
where
    T: CtSelect,
{
    core::array::from_fn(|i| T::ct_select(&a[i], &b[i], choice))
}

#[cfg(test)]
mod tests {
    use crate::Choice;

    // Note: this violates our own advice not to use these functions with e.g. `[u8; N]` but this
    // is just for testing purposes
    const EXAMPLE_A: [u8; 3] = [1, 2, 3];
    const EXAMPLE_B: [u8; 3] = [4, 5, 6];

    #[test]
    fn ct_assign() {
        let mut x = EXAMPLE_A;

        super::ct_assign(&mut x, &EXAMPLE_B, Choice::FALSE);
        assert_eq!(EXAMPLE_A, x);

        super::ct_assign(&mut x, &EXAMPLE_B, Choice::TRUE);
        assert_eq!(EXAMPLE_B, x);
    }

    #[test]
    fn ct_eq() {
        assert!(super::ct_eq(&EXAMPLE_A, &EXAMPLE_A).to_bool());
        assert!(!super::ct_eq(&EXAMPLE_A, &EXAMPLE_B).to_bool());
    }

    #[test]
    fn ct_select() {
        assert_eq!(
            EXAMPLE_A,
            super::ct_select(&EXAMPLE_A, &EXAMPLE_B, Choice::FALSE)
        );
        assert_eq!(
            EXAMPLE_B,
            super::ct_select(&EXAMPLE_A, &EXAMPLE_B, Choice::TRUE)
        );
    }
}
