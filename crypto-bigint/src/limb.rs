//! Big integers are represented as an array of smaller CPU word-size integers
//! called "limbs".

#![allow(clippy::derive_hash_xor_eq)]

mod add;
mod bit_and;
mod bit_or;
mod cmp;
mod encoding;
mod from;
mod mul;
mod sub;

#[cfg(feature = "rand")]
mod rand;

use core::fmt;
use subtle::{Choice, ConditionallySelectable};

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("this crate builds on 32-bit and 64-bit platforms only");

//
// 32-bit definitions
//

/// Size of the inner integer in bits.
#[cfg(target_pointer_width = "32")]
pub const BIT_SIZE: usize = 32;

/// Size of the inner integer in bytes.
#[cfg(target_pointer_width = "32")]
pub const BYTE_SIZE: usize = 4;

/// Inner integer type that the [`Limb`] newtype wraps.
#[cfg(target_pointer_width = "32")]
pub type Inner = u32;

/// SignedInner integer type that the [`Limb`] newtype wraps.
#[cfg(target_pointer_width = "32")]
pub(crate) type SignedInner = i32;

/// Wide integer type: double the width of [`Inner`].
#[cfg(target_pointer_width = "32")]
pub type Wide = u64;

/// SignedInner integer type: double the width of [`Limb`].
#[cfg(target_pointer_width = "32")]
pub type SignedWide = i64;

//
// 64-bit definitions
//

/// Size of the inner integer in bits.
#[cfg(target_pointer_width = "64")]
pub const BIT_SIZE: usize = 64;

/// Size of the inner integer in bytes.
#[cfg(target_pointer_width = "64")]
pub const BYTE_SIZE: usize = 8;

/// Inner integer type that the [`Limb`] newtype wraps.
#[cfg(target_pointer_width = "64")]
pub type Inner = u64;

/// SignedInner integer type that the [`Limb`] newtype wraps.
#[cfg(target_pointer_width = "64")]
pub(crate) type SignedInner = i64;

/// Wide integer type: double the width of [`Inner`].
#[cfg(target_pointer_width = "64")]
pub type Wide = u128;

/// SignedInner integer type: double the width of [`Limb`].
#[cfg(target_pointer_width = "64")]
pub type SignedWide = i128;

/// Highest bit in a [`Limb`].
pub(crate) const HI_BIT: usize = BIT_SIZE - 1;

/// Big integers are represented as an array of smaller CPU word-size integers
/// called "limbs".
#[derive(Copy, Clone, Debug, Default, Hash)]
#[repr(transparent)]
pub struct Limb(pub Inner);

impl Limb {
    /// The value `0`.
    pub const ZERO: Self = Limb(0);

    /// The value `1`.
    pub const ONE: Self = Limb(1);

    /// Maximum value this [`Limb`] can express.
    pub const MAX: Self = Limb(Inner::MAX);

    /// Return `a` if `c`!=0 or `b` if `c`==0.
    ///
    /// Const-friendly: we can't yet use `subtle` in `const fn` contexts.
    #[inline]
    pub(crate) const fn ct_select(a: Self, b: Self, c: Inner) -> Self {
        Self(a.0 ^ (c & (a.0 ^ b.0)))
    }
}

impl ConditionallySelectable for Limb {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(Inner::conditional_select(&a.0, &b.0, choice))
    }
}

impl fmt::Display for Limb {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}

impl fmt::LowerHex for Limb {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for Limb {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

#[cfg(feature = "zeroize")]
#[cfg_attr(docsrs, doc(cfg(feature = "zeroize")))]
impl zeroize::DefaultIsZeroes for Limb {}
