//! Big unsigned integers.

#![allow(
    clippy::needless_range_loop,
    clippy::many_single_char_names,
    clippy::derive_hash_xor_eq
)]

#[macro_use]
mod macros;

mod add;
mod add_mod;
mod cmp;
mod encoding;
mod from;
mod mul;
mod neg_mod;
mod shr;
mod sub;
mod sub_mod;

#[cfg(feature = "generic-array")]
mod array;

#[cfg(feature = "rand")]
mod rand;

use crate::{Concat, Encoding, Limb, Split};
use core::fmt;
use subtle::{Choice, ConditionallySelectable};

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

/// Big unsigned integer.
///
/// Generic over the given number of `LIMBS`
// TODO(tarcieri): make generic around a specified number of bits.
#[derive(Copy, Clone, Debug, Hash)]
pub struct UInt<const LIMBS: usize> {
    /// Inner limb array. Stored from least significant to most significant.
    limbs: [Limb; LIMBS],
}

impl<const LIMBS: usize> UInt<LIMBS> {
    /// The value `0`.
    pub const ZERO: Self = Self::from_u8(0);

    /// The value `1`.
    pub const ONE: Self = Self::from_u8(1);

    /// Maximum value this [`UInt`] can express.
    pub const MAX: Self = Self {
        limbs: [Limb::MAX; LIMBS],
    };

    /// Const-friendly [`UInt`] constructor.
    pub const fn new(limbs: [Limb; LIMBS]) -> Self {
        Self { limbs }
    }

    /// Borrow the limbs of this [`UInt`].
    // TODO(tarcieri): eventually phase this out?
    pub const fn limbs(&self) -> &[Limb; LIMBS] {
        &self.limbs
    }

    /// Convert this [`UInt`] into its inner limbs.
    // TODO(tarcieri): eventually phase this out?
    pub const fn into_limbs(self) -> [Limb; LIMBS] {
        self.limbs
    }
}

// TODO(tarcieri): eventually phase this out?
impl<const LIMBS: usize> AsRef<[Limb]> for UInt<LIMBS> {
    fn as_ref(&self) -> &[Limb] {
        self.limbs()
    }
}

// TODO(tarcieri): eventually phase this out?
impl<const LIMBS: usize> AsMut<[Limb]> for UInt<LIMBS> {
    fn as_mut(&mut self) -> &mut [Limb] {
        &mut self.limbs
    }
}

impl<const LIMBS: usize> ConditionallySelectable for UInt<LIMBS> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut limbs = [Limb::ZERO; LIMBS];

        for i in 0..LIMBS {
            limbs[i] = Limb::conditional_select(&a.limbs[i], &b.limbs[i], choice);
        }

        Self { limbs }
    }
}

impl<const LIMBS: usize> Default for UInt<LIMBS> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const LIMBS: usize> fmt::Display for UInt<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(self, f)
    }
}

impl<const LIMBS: usize> fmt::LowerHex for UInt<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for limb in self.limbs.iter().rev() {
            fmt::LowerHex::fmt(limb, f)?;
        }
        Ok(())
    }
}

impl<const LIMBS: usize> fmt::UpperHex for UInt<LIMBS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for limb in self.limbs.iter().rev() {
            fmt::UpperHex::fmt(limb, f)?;
        }
        Ok(())
    }
}

#[cfg(feature = "zeroize")]
#[cfg_attr(docsrs, doc(cfg(feature = "zeroize")))]
impl<const LIMBS: usize> Zeroize for UInt<LIMBS> {
    fn zeroize(&mut self) {
        self.limbs.zeroize();
    }
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
impl_uint_aliases! {
    (U64, 64, "64-bit"),
    (U128, 128, "128-bit"),
    (U192, 192, "192-bit"),
    (U256, 256, "256-bit"),
    (U384, 384, "384-bit"),
    (U448, 448, "448-bit"),
    (U512, 512, "512-bit"),
    (U768, 768, "768-bit"),
    (U896, 896, "896-bit"),
    (U1024, 1024, "1024-bit"),
    (U1536, 1536, "1536-bit"),
    (U1792, 1792, "1792-bit"),
    (U2048, 2048, "2048-bit"),
    (U3072, 3072, "3072-bit"),
    (U3584, 3584, "3584-bit"),
    (U4096, 4096, "4096-bit"),
    (U6144, 6144, "6144-bit"),
    (U8192, 8192, "8192-bit")
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
impl_concat! {
    (U64, 64),
    (U128, 128),
    (U192, 192),
    (U256, 256),
    (U384, 384),
    (U448, 448),
    (U512, 512),
    (U768, 768),
    (U896, 896),
    (U1024, 1024),
    (U1536, 1536),
    (U1792, 1792),
    (U2048, 2048),
    (U3072, 3072),
    (U4096, 4096)
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
impl_split! {
    (U128, 128),
    (U192, 192),
    (U256, 256),
    (U384, 384),
    (U448, 448),
    (U512, 512),
    (U768, 768),
    (U896, 896),
    (U1024, 1024),
    (U1536, 1536),
    (U1792, 1792),
    (U2048, 2048),
    (U3072, 3072),
    (U3584, 3584),
    (U4096, 4096),
    (U6144, 6144),
    (U8192, 8192)
}

#[cfg(test)]
mod tests {
    use crate::{Concat, Split, U128, U64};
    use subtle::ConditionallySelectable;

    #[test]
    #[cfg(feature = "alloc")]
    fn display() {
        let hex = "AAAAAAAABBBBBBBBCCCCCCCCDDDDDDDD";
        let n = U128::from_be_hex(hex);

        use alloc::string::ToString;
        assert_eq!(hex, n.to_string());
    }

    #[test]
    fn conditional_select() {
        let a = U128::from_be_hex("00002222444466668888AAAACCCCEEEE");
        let b = U128::from_be_hex("11113333555577779999BBBBDDDDFFFF");

        let select_0 = U128::conditional_select(&a, &b, 0.into());
        assert_eq!(a, select_0);

        let select_1 = U128::conditional_select(&a, &b, 1.into());
        assert_eq!(b, select_1);
    }

    #[test]
    fn concat() {
        let hi = U64::from_u64(0x0011223344556677);
        let lo = U64::from_u64(0x8899aabbccddeeff);
        assert_eq!(
            hi.concat(&lo),
            U128::from_be_hex("00112233445566778899aabbccddeeff")
        );
    }

    #[test]
    fn split() {
        let (hi, lo) = U128::from_be_hex("00112233445566778899aabbccddeeff").split();
        assert_eq!(hi, U64::from_u64(0x0011223344556677));
        assert_eq!(lo, U64::from_u64(0x8899aabbccddeeff));
    }
}
