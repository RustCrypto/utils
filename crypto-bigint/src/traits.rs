//! Traits provided by this crate

/// Concatenate two numbers into a "wide" twice-width value, using the `rhs`
/// value as the least significant value.
pub trait Concat<Rhs = Self> {
    /// Concatenated output: twice the width of `Self`.
    type Output;

    /// Concate the two values, with `self` as most significant and `rhs` as
    /// the least significant.
    fn concat(&self, rhs: &Self) -> Self::Output;
}

/// Number of bits required to express a given big integer.
pub trait NumBits {
    /// Number of bits required to express this integer.
    const NUM_BITS: usize;
}

/// Number of bytes required to express a given big integer.
pub trait NumBytes {
    /// Number of bytes required to express this integer.
    const NUM_BYTES: usize;
}

/// Split a number in half, returning the most significant half followed by
/// the least significant.
pub trait Split<Rhs = Self> {
    /// Split output: high/low components of the value.
    type Output;

    /// Split this number in half, returning its high and low components
    /// respectively.
    fn split(&self) -> (Self::Output, Self::Output);
}
