//! Traits provided by this crate

use crate::Limb;
use core::fmt::Debug;
use subtle::{
    Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
};

#[cfg(feature = "rand")]
use rand_core::{CryptoRng, RngCore};

/// Integer type.
pub trait Integer:
    'static
    + AsRef<[Limb]>
    + Copy
    + ConditionallySelectable
    + ConstantTimeEq
    + ConstantTimeGreater
    + ConstantTimeLess
    + Debug
    + Default
    + Encoding
    + Eq
    + From<u64>
    + Ord
    + Send
    + Sized
    + Sync
{
    /// The value `0`.
    const ZERO: Self;

    /// The value `1`.
    const ONE: Self;

    /// Maximum value this integer can express.
    const MAX: Self;

    /// Is this integer value equal to zero?
    fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }

    /// Is this integer value an odd number?
    fn is_odd(&self) -> Choice;

    /// Is this integer value an even number?
    fn is_even(&self) -> Choice {
        !self.is_odd()
    }
}

/// Random number generation support.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub trait Random: Sized {
    /// Generate a cryptographically secure random value.
    fn random(rng: impl CryptoRng + RngCore) -> Self;
}

/// Modular random number generation support.
#[cfg(feature = "rand")]
#[cfg_attr(docsrs, doc(cfg(feature = "rand")))]
pub trait RandomMod: Sized {
    /// Generate a cryptographically secure random number which is less than
    /// a given `modulus`.
    ///
    /// This function uses rejection sampling, a method which produces an
    /// unbiased distribution of in-range values provided the underlying
    /// [`CryptoRng`] is unbiased, but runs in variable-time.
    ///
    /// The variable-time nature of the algorithm should not pose a security
    /// issue so long as the underlying random number generator is truly a
    /// [`CryptoRng`], where previous outputs are unrelated to subsequent
    /// outputs and do not reveal information about the RNG's internal state.
    fn random_mod(rng: impl CryptoRng + RngCore, modulus: &Self) -> Self;
}

/// Compute `self + rhs mod p`.
pub trait AddMod<Rhs = Self> {
    /// Output type.
    type Output;

    /// Compute `self + rhs mod p`.
    ///
    /// Assumes `self` and `rhs` are `< p`.
    fn add_mod(&self, rhs: &Rhs, p: &Self) -> Self::Output;
}

/// Compute `self - rhs mod p`.
pub trait SubMod<Rhs = Self> {
    /// Output type.
    type Output;

    /// Compute `self - rhs mod p`.
    ///
    /// Assumes `self` and `rhs` are `< p`.
    fn sub_mod(&self, rhs: &Rhs, p: &Self) -> Self::Output;
}

/// Compute `-self mod p`.
pub trait NegMod {
    /// Output type.
    type Output;

    /// Compute `-self mod p`.
    #[must_use]
    fn neg_mod(&self, p: &Self) -> Self::Output;
}

/// Compute `self * rhs mod p`.
///
/// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
pub trait MulMod<Rhs = Self> {
    /// Output type.
    type Output;

    /// Compute `self * rhs mod p`.
    ///
    /// Requires `p_inv = -(p^{-1} mod 2^{BITS}) mod 2^{BITS}` to be provided for efficiency.
    fn mul_mod(&self, rhs: &Rhs, p: &Self, p_inv: Limb) -> Self::Output;
}

/// Concatenate two numbers into a "wide" twice-width value, using the `rhs`
/// value as the least significant value.
pub trait Concat<Rhs = Self> {
    /// Concatenated output: twice the width of `Self`.
    type Output;

    /// Concate the two values, with `self` as most significant and `rhs` as
    /// the least significant.
    fn concat(&self, rhs: &Self) -> Self::Output;
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

/// Encoding support.
pub trait Encoding: Sized {
    /// Size of this integer in bits.
    const BIT_SIZE: usize;

    /// Size of this integer in bytes.
    const BYTE_SIZE: usize;

    /// Byte array representation.
    type Repr: AsRef<[u8]> + AsMut<[u8]> + Copy + Clone + Sized;

    /// Decode from big endian bytes.
    fn from_be_bytes(bytes: Self::Repr) -> Self;

    /// Decode from little endian bytes.
    fn from_le_bytes(bytes: Self::Repr) -> Self;

    /// Encode to big endian bytes.
    fn to_be_bytes(&self) -> Self::Repr;

    /// Encode to little endian bytes.
    fn to_le_bytes(&self) -> Self::Repr;
}
