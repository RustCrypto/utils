//! Limb newtype.

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

/// Size of the inner integer in bits
#[cfg(target_pointer_width = "32")]
pub(crate) const BIT_SIZE: usize = 32;
#[cfg(target_pointer_width = "64")]
pub(crate) const BIT_SIZE: usize = 64;

/// Size of the inner integer in bytes
#[cfg(target_pointer_width = "32")]
pub(crate) const BYTE_SIZE: usize = 4;
#[cfg(target_pointer_width = "64")]
pub(crate) const BYTE_SIZE: usize = 8;

/// Inner integer type.
// TODO(tarcieri): expose this?
#[cfg(target_pointer_width = "32")]
pub(crate) type Inner = u32;
#[cfg(target_pointer_width = "64")]
pub(crate) type Inner = u64;

/// Wide integer type: double the width of [`Inner`].
#[cfg(target_pointer_width = "32")]
pub(crate) type Wide = u64;
#[cfg(target_pointer_width = "64")]
pub(crate) type Wide = u128;

/// Big integers are represented as an array of smaller CPU word-size integers
/// called "limbs".
#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct Limb(pub Inner);

impl Limb {
    /// The value `0`.
    pub const ZERO: Self = Limb(0);

    /// The value `1`.
    pub const ONE: Self = Limb(1);

    /// Maximum value this [`Limb`] can express.
    pub const MAX: Self = Limb(Inner::MAX);
}

impl ConditionallySelectable for Limb {
    #[inline]
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Limb(Inner::conditional_select(&a.0, &b.0, choice))
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
