use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// Constant-time analogue of `bool` providing a "best effort" optimization barrier.
///
/// Attempts to hint to the compiler and its codegen backends that optimizations should not be
/// applied which depend on a value prior to accessing it though [`BlackBox::get`].
///
/// This is used as a "belt-and-suspenders" defense in addition to rhs mechanisms like
/// constant-time predication intrinsics, and is never expected to be the only line of defense.
#[derive(Copy, Clone, Debug)]
pub struct Choice(u8);

impl Choice {
    /// The falsy value.
    pub const FALSE: Self = Self(0);

    /// The truthy value.
    pub const TRUE: Self = Self(1);

    /// Create a new [`Choice`] from the given `u8` value, which should be either `0` or `1`.
    #[inline]
    pub const fn new(value: u8) -> Self {
        debug_assert!(value == 0 || value == 1, "Choice::new accepts only 0 or 1");
        Self(value)
    }

    /// Convert `Choice` into a `bool`.
    ///
    /// # ⚠️ Security Note
    ///
    /// This is intended to be used as either the one and only branch at the end of a constant-time
    /// operation to e.g. differentiate between success and failure, or in contexts where
    /// constant-time doesn't matter, e.g. vartime code that operates on "maybe secret" types which
    /// aren't secrets in a particular context.
    // TODO(tarcieri): `const fn` when MSRV 1.86
    pub fn to_bool(self) -> bool {
        self.to_u8() != 0
    }

    /// Convert [`Choice`] to a `u8`, attempting to apply an optimization barrier if possible.
    // TODO(tarcieri): `const fn` when MSRV 1.86
    pub fn to_u8(self) -> u8 {
        // `black_box` is documented as working on a "best effort" basis. That's fine, this type is
        // likewise documented as only working on a "best effort" basis itself. The only way we
        // rely on `black_box` for correctness is it behaving as the identity function.
        core::hint::black_box(self.0)
    }

    /// Apply an `and` conditional to the given [`Choice`]s.
    #[inline]
    pub const fn and(self, rhs: Choice) -> Choice {
        Self(self.0 & rhs.0)
    }

    /// Apply an `or` conditional to the given [`Choice`]s.
    #[inline]
    pub const fn or(self, rhs: Choice) -> Choice {
        Self(self.0 | rhs.0)
    }

    /// Apply an `xor` conditional to the given [`Choice`]s.
    #[inline]
    pub const fn xor(self, rhs: Choice) -> Choice {
        Self(self.0 ^ rhs.0)
    }

    /// Compute the boolean inverse of `self`.
    #[inline]
    pub const fn not(self) -> Choice {
        // NOTE: assumes self.0 is `0` or `1` as checked in constructor
        Self(self.0 ^ 1)
    }
}

impl BitAnd for Choice {
    type Output = Choice;

    #[inline]
    fn bitand(self, rhs: Choice) -> Choice {
        self.and(rhs)
    }
}

impl BitAndAssign for Choice {
    #[inline]
    fn bitand_assign(&mut self, rhs: Choice) {
        *self = *self & rhs;
    }
}

impl BitOr for Choice {
    type Output = Choice;

    #[inline]
    fn bitor(self, rhs: Choice) -> Choice {
        self.or(rhs)
    }
}

impl BitOrAssign for Choice {
    #[inline]
    fn bitor_assign(&mut self, rhs: Choice) {
        *self = *self | rhs;
    }
}

impl BitXor for Choice {
    type Output = Choice;

    #[inline]
    fn bitxor(self, rhs: Choice) -> Choice {
        (self.0 ^ rhs.0).into()
    }
}

impl BitXorAssign for Choice {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Choice) {
        *self = *self ^ rhs;
    }
}

impl From<u8> for Choice {
    fn from(value: u8) -> Self {
        Self::new(value)
    }
}

impl From<Choice> for u8 {
    fn from(choice: Choice) -> u8 {
        choice.to_u8()
    }
}

impl From<Choice> for bool {
    fn from(choice: Choice) -> bool {
        choice.to_bool()
    }
}

impl Not for Choice {
    type Output = Choice;

    #[inline]
    fn not(self) -> Choice {
        self.not()
    }
}

#[cfg(test)]
mod tests {
    use super::Choice;

    #[test]
    fn to_bool() {
        assert_eq!(Choice::new(0).to_bool(), false);
        assert_eq!(Choice::new(1).to_bool(), true);
    }

    #[test]
    fn to_u8() {
        assert_eq!(Choice::new(0).to_u8(), 0);
        assert_eq!(Choice::new(1).to_u8(), 1);
    }

    #[test]
    fn and() {
        assert_eq!((Choice::new(0) & Choice::new(0)).to_u8(), 0);
        assert_eq!((Choice::new(1) & Choice::new(0)).to_u8(), 0);
        assert_eq!((Choice::new(0) & Choice::new(1)).to_u8(), 0);
        assert_eq!((Choice::new(1) & Choice::new(1)).to_u8(), 1);
    }

    #[test]
    fn or() {
        assert_eq!((Choice::new(0) | Choice::new(0)).to_u8(), 0);
        assert_eq!((Choice::new(1) | Choice::new(0)).to_u8(), 1);
        assert_eq!((Choice::new(0) | Choice::new(1)).to_u8(), 1);
        assert_eq!((Choice::new(1) | Choice::new(1)).to_u8(), 1);
    }

    #[test]
    fn xor() {
        assert_eq!((Choice::new(0) ^ Choice::new(0)).to_u8(), 0);
        assert_eq!((Choice::new(1) ^ Choice::new(0)).to_u8(), 1);
        assert_eq!((Choice::new(0) ^ Choice::new(1)).to_u8(), 1);
        assert_eq!((Choice::new(1) ^ Choice::new(1)).to_u8(), 0);
    }

    #[test]
    fn not() {
        assert_eq!(Choice::new(0).not().to_u8(), 1);
        assert_eq!(Choice::new(1).not().to_u8(), 0);
    }
}
