//! Slice helpers: generic selection/equality operations for slices.
//!
//! Use these in the event there isn't an impl of a given trait for `[T]`. If there is, however,
//! you should prefer that for performance reasons.

use crate::{Choice, CtAssign, CtEq};

/// Generic implementation of conditional assignment for slices which works with any type which
/// impls `CtSelect`. Useful in the event there isn't a `CtAssign` impl for `[T]`.
///
/// Assigns `src` to `dst` in constant-time when `choice` is [`Choice::TRUE`], like a conditional
/// version of `[T]::copy_from_slice`.
///
/// Unfortunately we can't provide this as a trait impl without specialization, since it would
/// overlap with the optimized type-specific impls we provide.
///
/// # Panics
/// - If the two slices have unequal lengths
#[inline]
pub fn ct_assign<T>(dst: &mut [T], src: &[T], choice: Choice)
where
    T: CtAssign,
{
    assert_eq!(
        dst.len(),
        src.len(),
        "source slice length ({}) does not match destination slice length ({})",
        src.len(),
        dst.len()
    );

    for (a, b) in dst.iter_mut().zip(src) {
        a.ct_assign(b, choice)
    }
}

/// Generic implementation of constant-time equality testing for slices which works with any type
/// which impls `CtEq`. Useful in the event there isn't a `CtEq` impl for `[T]`.
// NOTE: `cfg` gated because it uses the `CtEq` impl on `usize`
#[cfg(any(
    target_pointer_width = "16",
    target_pointer_width = "32",
    target_pointer_width = "64"
))]
#[inline]
pub fn ct_eq<T>(a: &[T], b: &[T]) -> Choice
where
    T: CtEq,
{
    let mut ret = a.len().ct_eq(&b.len());
    for (a, b) in a.iter().zip(b.iter()) {
        ret &= a.ct_eq(b);
    }
    ret
}

#[cfg(test)]
mod tests {
    use crate::Choice;

    // Note: this violates our own advice not to use these functions with e.g. `[u8]` but this
    // is just for testing purposes
    const EXAMPLE_A: &[u8] = &[1, 2, 3];
    const EXAMPLE_B: &[u8] = &[4, 5, 6];

    #[test]
    fn ct_assign() {
        let mut x = [0u8; 3];

        super::ct_assign(&mut x, EXAMPLE_A, Choice::FALSE);
        assert_eq!([0u8; 3], x);

        super::ct_assign(&mut x, EXAMPLE_A, Choice::TRUE);
        assert_eq!(EXAMPLE_A, x);
    }

    #[test]
    fn ct_eq() {
        assert!(super::ct_eq(EXAMPLE_A, EXAMPLE_A).to_bool());
        assert!(!super::ct_eq(EXAMPLE_A, EXAMPLE_B).to_bool());
    }
}
