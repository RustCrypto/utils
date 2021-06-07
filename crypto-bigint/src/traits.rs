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

/// Encoding support.
pub trait Encoding: Sized {
    /// Size of this integer in bits.
    const BIT_SIZE: usize;

    /// Size of this integer in bytes.
    const BYTE_SIZE: usize;

    /// Byte array representation.
    type Repr: Copy + Clone + AsRef<[u8]> + AsMut<[u8]> + Sized;

    /// Decode from big endian bytes.
    fn from_be_bytes(bytes: Self::Repr) -> Self;

    /// Decode from little endian bytes.
    fn from_le_bytes(bytes: Self::Repr) -> Self;

    /// Encode to big endian bytes.
    fn to_be_bytes(&self) -> Self::Repr;

    /// Encode to little endian bytes.
    fn to_le_bytes(&self) -> Self::Repr;
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
