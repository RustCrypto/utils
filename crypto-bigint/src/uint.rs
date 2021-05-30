//! Big unsigned integers.

#![allow(clippy::needless_range_loop, clippy::many_single_char_names)]

mod decoder;

use self::decoder::Decoder;
use crate::{limb, Concat, Limb, NumBits, NumBytes, Split, LIMB_BYTES};
use core::fmt;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// Big unsigned integer.
///
/// Generic over the given number of `LIMBS`
// TODO(tarcieri): make generic around a specified number of bits.
#[derive(Copy, Clone, Debug)]
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

    /// Create a [`UInt`] from a `u8` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u8>` when stable
    pub const fn from_u8(n: u8) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u16` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u16>` when stable
    pub const fn from_u16(n: u16) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u32` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u32>` when stable
    pub const fn from_u32(n: u32) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u64` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u64>` when stable
    #[cfg(target_pointer_width = "32")]
    pub const fn from_u64(n: u64) -> Self {
        const_assert!(LIMBS >= 2, "number of limbs must be two or greater");
        let mut limbs = [0; LIMBS];
        limbs[0] = (n & 0xFFFFFFFF) as u32;
        limbs[1] = (n >> 32) as u32;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u64` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u64>` when stable
    #[cfg(target_pointer_width = "64")]
    pub const fn from_u64(n: u64) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u128` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u128>` when stable
    pub const fn from_u128(n: u128) -> Self {
        const_assert!(
            LIMBS >= (16 / LIMB_BYTES),
            "number of limbs must be greater than zero"
        );

        let lo = Self::from_u64((n & 0xffff_ffff_ffff_ffff) as u64);
        let hi = Self::from_u64((n >> 64) as u64);

        let mut i = 0;
        let mut limbs = [0; LIMBS];

        while i < LIMBS {
            limbs[i] = lo.limbs[i];
            limbs[i + LIMBS] = hi.limbs[i];
            i += 1;
        }

        Self { limbs }
    }

    /// Create a new [`UInt`] from the provided big endian bytes.
    pub const fn from_be_bytes(bytes: &[u8]) -> Self {
        const_assert!(
            bytes.len() == LIMB_BYTES * LIMBS,
            "bytes are not the expected size"
        );

        let mut decoder = Decoder::new();
        let mut i = 0;

        while i < LIMB_BYTES * LIMBS {
            i += 1;
            decoder = decoder.add_byte(bytes[bytes.len() - i]);
        }

        decoder.finish()
    }

    /// Create a new [`UInt`] from the provided big endian hex string.
    pub const fn from_be_hex(hex: &str) -> Self {
        let bytes = hex.as_bytes();

        const_assert!(
            bytes.len() == LIMB_BYTES * LIMBS * 2,
            "hex string is not the expected size"
        );

        let mut decoder = Decoder::new();
        let mut i = 0;

        while i < LIMB_BYTES * LIMBS * 2 {
            i += 2;
            let offset = bytes.len() - i;
            let byte = decode_hex_byte([bytes[offset], bytes[offset + 1]]);
            decoder = decoder.add_byte(byte);
        }

        decoder.finish()
    }

    /// Create a new [`UInt`] from the provided little endian bytes.
    pub const fn from_le_bytes(bytes: &[u8]) -> Self {
        const_assert!(
            bytes.len() == LIMB_BYTES * LIMBS,
            "bytes are not the expected size"
        );

        let mut decoder = Decoder::new();
        let mut i = 0;

        while i < LIMB_BYTES * LIMBS {
            decoder = decoder.add_byte(bytes[i]);
            i += 1;
        }

        decoder.finish()
    }

    /// Create a new [`UInt`] from the provided little endian hex string.
    pub const fn from_le_hex(hex: &str) -> Self {
        let bytes = hex.as_bytes();

        const_assert!(
            bytes.len() == LIMB_BYTES * LIMBS * 2,
            "bytes are not the expected size"
        );

        let mut decoder = Decoder::new();
        let mut i = 0;

        while i < LIMB_BYTES * LIMBS * 2 {
            let byte = decode_hex_byte([bytes[i], bytes[i + 1]]);
            decoder = decoder.add_byte(byte);
            i += 2;
        }

        decoder.finish()
    }

    /// Serialize this [`UInt`] as big-endian, writing it into the provided
    /// byte slice.
    // TODO(tarcieri): const generics support when possible
    #[cfg(feature = "generic-array")]
    #[inline]
    pub(crate) fn to_be_bytes(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), LIMB_BYTES * LIMBS);

        for (src, dst) in self
            .limbs
            .iter()
            .rev()
            .zip(out.chunks_exact_mut(LIMB_BYTES))
        {
            dst.copy_from_slice(&src.to_be_bytes());
        }
    }

    /// Serialize this [`UInt`] as little-endian, writing it into the provided
    /// byte slice.
    // TODO(tarcieri): const generics support when possible
    #[cfg(feature = "generic-array")]
    #[inline]
    pub(crate) fn to_le_bytes(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), LIMB_BYTES * LIMBS);

        for (src, dst) in self.limbs.iter().zip(out.chunks_exact_mut(LIMB_BYTES)) {
            dst.copy_from_slice(&src.to_le_bytes());
        }
    }

    /// Borrow the limbs of this [`UInt`].
    pub const fn limbs(&self) -> &[Limb; LIMBS] {
        &self.limbs
    }

    /// Computes `a + b + carry`, returning the result along with the new carry.
    #[inline(always)]
    pub const fn adc(&self, rhs: &Self, mut carry: Limb) -> (Self, Limb) {
        let mut limbs = [0; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let (w, c) = limb::adc(self.limbs[i], rhs.limbs[i], carry);
            limbs[i] = w;
            carry = c;
            i += 1;
        }

        (Self { limbs }, carry)
    }

    /// Computes `a - (b + borrow)`, returning the result along with the new borrow.
    #[inline(always)]
    pub const fn sbb(&self, rhs: &Self, mut borrow: Limb) -> (Self, Limb) {
        let mut limbs = [0; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let (w, b) = limb::sbb(self.limbs[i], rhs.limbs[i], borrow);
            limbs[i] = w;
            borrow = b;
            i += 1;
        }

        (Self { limbs }, borrow)
    }

    /// Compute "wide" multiplication, with a product twice the size of the input.
    // TODO(tarcieri): use `concat` (or replacement) when traits are const-friendly
    pub const fn mul_wide(&self, rhs: &Self) -> (Self, Self) {
        let mut i = 0;
        let mut lo = Self::ZERO;
        let mut hi = Self::ZERO;

        // Schoolbook multiplication.
        // TODO(tarcieri): use Karatsuba for better performance?
        while i < LIMBS {
            let mut j = 0;
            let mut carry = 0;

            while j < LIMBS {
                let k = i + j;

                if k >= LIMBS {
                    let (n, c) = limb::mac(hi.limbs[k - LIMBS], self.limbs[i], rhs.limbs[j], carry);
                    hi.limbs[k - LIMBS] = n;
                    carry = c;
                } else {
                    let (n, c) = limb::mac(lo.limbs[k], self.limbs[i], rhs.limbs[j], carry);
                    lo.limbs[k] = n;
                    carry = c;
                }

                j += 1;
            }

            hi.limbs[i + j - LIMBS] = carry;
            i += 1;
        }

        (hi, lo)
    }

    /// Perform wrapping addition, discarding overflow.
    pub const fn wrapping_add(&self, rhs: &Self) -> Self {
        self.adc(rhs, 0).0
    }

    /// Perform wrapping subtraction, discarding underflow and wrapping around
    /// the boundary of the type.
    pub const fn wrapping_sub(&self, rhs: &Self) -> Self {
        self.sbb(rhs, 0).0
    }

    /// Perform wrapping multiplication, discarding overflow.
    pub const fn wrapping_mul(&self, rhs: &Self) -> Self {
        self.mul_wide(rhs).0
    }

    /// Perform checked addition, returning [`CtOption`] only if the operation
    /// did not overflow.
    pub fn checked_add(&self, rhs: &Self) -> CtOption<Self> {
        let (result, carry) = self.adc(rhs, 0);
        CtOption::new(result, carry.ct_eq(&0))
    }

    /// Perform checked subtraction, returning [`CtOption`] only if the operation
    /// did not underflow.
    pub fn checked_sub(&self, rhs: &Self) -> CtOption<Self> {
        let (result, underflow) = self.sbb(rhs, 0);
        CtOption::new(result, underflow.ct_eq(&0))
    }

    /// Perform checked multiplication, returning [`CtOption`] only if the
    /// operation did not overflow.
    pub fn checked_mul(&self, rhs: &Self) -> CtOption<Self> {
        let (hi, lo) = self.mul_wide(rhs);
        CtOption::new(lo, hi.is_zero())
    }

    /// Square self, returning a "wide" result.
    pub fn square(&self) -> <Self as Concat>::Output
    where
        Self: Concat,
    {
        let (hi, lo) = self.mul_wide(self);
        hi.concat(&lo)
    }

    /// Determine if this [`UInt`] is equal to zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }
}

impl<const LIMBS: usize> AsRef<[Limb]> for UInt<LIMBS> {
    fn as_ref(&self) -> &[Limb] {
        self.limbs()
    }
}

impl<const LIMBS: usize> ConditionallySelectable for UInt<LIMBS> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut limbs = [0; LIMBS];

        for i in 0..LIMBS {
            limbs[i] = Limb::conditional_select(&a.limbs[0], &b.limbs[0], choice);
        }

        Self { limbs }
    }
}

impl<const LIMBS: usize> ConstantTimeEq for UInt<LIMBS> {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.limbs
            .iter()
            .zip(other.limbs.iter())
            .fold(Choice::from(1), |acc, (a, b)| acc & a.ct_eq(b))
    }
}

impl<const LIMBS: usize> Default for UInt<LIMBS> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const LIMBS: usize> From<u8> for UInt<LIMBS> {
    fn from(n: u8) -> Self {
        // TODO(tarcieri): const where clause when possible
        debug_assert!(LIMBS > 0, "limbs must be non-zero");
        Self::from_u8(n)
    }
}

impl<const LIMBS: usize> From<u16> for UInt<LIMBS> {
    fn from(n: u16) -> Self {
        // TODO(tarcieri): const where clause when possible
        debug_assert!(LIMBS > 0, "limbs must be non-zero");
        Self::from_u16(n)
    }
}

impl<const LIMBS: usize> From<u32> for UInt<LIMBS> {
    fn from(n: u32) -> Self {
        // TODO(tarcieri): const where clause when possible
        debug_assert!(LIMBS > 0, "limbs must be non-zero");
        Self::from_u32(n)
    }
}

impl<const LIMBS: usize> From<u64> for UInt<LIMBS> {
    fn from(n: u64) -> Self {
        // TODO(tarcieri): const where clause when possible
        debug_assert!(LIMBS > (8 / LIMB_BYTES), "not enough limbs");
        Self::from_u64(n)
    }
}

impl<const LIMBS: usize> From<u128> for UInt<LIMBS> {
    fn from(n: u128) -> Self {
        // TODO(tarcieri): const where clause when possible
        debug_assert!(LIMBS > (16 / LIMB_BYTES), "not enough limbs");
        Self::from_u128(n)
    }
}

#[cfg(target_pointer_width = "32")]
impl From<U64> for u64 {
    fn from(n: U64) -> u64 {
        (n.limbs[0] as u64) | ((n.limbs[1] as u64) << 32)
    }
}

#[cfg(target_pointer_width = "64")]
impl From<U64> for u64 {
    fn from(n: U64) -> u64 {
        n.limbs[0]
    }
}

impl From<U128> for u128 {
    fn from(n: U128) -> u128 {
        let (hi, lo) = n.split();
        (u64::from(hi) as u128) << 64 | (u64::from(lo) as u128)
    }
}

impl<const LIMBS: usize> PartialEq for UInt<LIMBS> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl<const LIMBS: usize> Eq for UInt<LIMBS> {}

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

/// Decode a single byte encoded as two hexadecimal characters.
const fn decode_hex_byte(bytes: [u8; 2]) -> u8 {
    let mut i = 0;
    let mut result = 0u8;

    while i < 2 {
        result <<= 4;
        result |= match bytes[i] {
            b @ b'0'..=b'9' => b - b'0',
            b @ b'a'..=b'f' => 10 + b - b'a',
            b @ b'A'..=b'F' => 10 + b - b'A',
            b => {
                const_assert!(
                    matches!(b, b'0'..=b'9' | b'a' ..= b'f' | b'A'..=b'F'),
                    "invalid hex byte"
                );
                0
            }
        };

        i += 1;
    }

    result
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
macro_rules! impl_biguint_aliases {
    ($(($name:ident, $bits:expr, $doc:expr)),+) => {
        $(
            #[doc = $doc]
            #[doc="unsigned big integer"]
            pub type $name = UInt<{nlimbs!($bits)}>;

            impl NumBits for $name {
                const NUM_BITS: usize = $bits;
            }

            impl NumBytes for $name {
                const NUM_BYTES: usize = $bits / 8;
            }
        )+
     };
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
macro_rules! impl_concat {
    ($(($name:ident, $bits:expr)),+) => {
        $(
            impl Concat for $name {
                type Output = UInt<{nlimbs!($bits) * 2}>;

                fn concat(&self, rhs: &Self) -> Self::Output {
                    let mut output = Self::Output::default();
                    let (lo, hi) = output.limbs.split_at_mut(self.limbs.len());
                    lo.copy_from_slice(&rhs.limbs);
                    hi.copy_from_slice(&self.limbs);
                    output
                }
            }

            impl From<($name, $name)> for UInt<{nlimbs!($bits) * 2}> {
                fn from(nums: ($name, $name)) -> UInt<{nlimbs!($bits) * 2}> {
                    nums.0.concat(&nums.1)
                }
            }
        )+
     };
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
macro_rules! impl_split {
    ($(($name:ident, $bits:expr)),+) => {
        $(
            impl Split for $name {
                type Output = UInt<{nlimbs!($bits) / 2}>;

                fn split(&self) -> (Self::Output, Self::Output) {
                    let mut hi_out = Self::Output::default();
                    let mut lo_out = Self::Output::default();
                    let (lo_in, hi_in) = self.limbs.split_at(self.limbs.len() / 2);
                    hi_out.limbs.copy_from_slice(&hi_in);
                    lo_out.limbs.copy_from_slice(&lo_in);
                    (hi_out, lo_out)
                }
            }
        )+
     };
}

// TODO(tarcieri): use `const_evaluatable_checked` when stable to make generic around bits.
impl_biguint_aliases! {
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
    use super::{U128, U64};
    use crate::{Concat, Limb, Split};
    use hex_literal::hex;

    // 2-limb example that's twice as wide as the native word size
    #[cfg(target_pointer_width = "64")]
    use super::U128 as UIntEx;
    #[cfg(target_pointer_width = "32")]
    use super::U64 as UIntEx;

    /// Byte array that corresponds to `UIntEx`
    #[cfg(feature = "generic-array")]
    use crate::ArrayEncoding;
    #[cfg(feature = "generic-array")]
    type ByteArray = crate::ByteArray<UIntEx>;

    #[test]
    fn from_u8() {
        let n = UIntEx::from_u8(42u8);
        assert_eq!(&n.limbs, &[42, 0]);
    }

    #[test]
    fn from_u16() {
        let n = UIntEx::from_u16(42u16);
        assert_eq!(&n.limbs, &[42, 0]);
    }

    #[test]
    fn from_u64() {
        let n = UIntEx::from_u64(42u64);
        assert_eq!(&n.limbs, &[42, 0]);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn from_be_bytes() {
        let bytes = hex!("0011223344556677");
        let n = UIntEx::from_be_bytes(&bytes);
        assert_eq!(&n.limbs, &[0x44556677, 0x00112233]);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn from_be_bytes() {
        let bytes = hex!("00112233445566778899aabbccddeeff");
        let n = UIntEx::from_be_bytes(&bytes);
        assert_eq!(&n.limbs, &[0x8899aabbccddeeff, 0x0011223344556677]);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn from_le_bytes() {
        let bytes = hex!("7766554433221100");
        let n = UIntEx::from_le_bytes(&bytes);
        assert_eq!(&n.limbs, &[0x44556677, 0x00112233]);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn from_le_bytes() {
        let bytes = hex!("ffeeddccbbaa99887766554433221100");
        let n = UIntEx::from_le_bytes(&bytes);
        assert_eq!(&n.limbs, &[0x8899aabbccddeeff, 0x0011223344556677]);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "32")]
    fn from_be_byte_array() {
        let n = UIntEx::from_be_byte_array(&hex!("0011223344556677").into());
        assert_eq!(&n.limbs, &[0x44556677, 0x00112233]);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "64")]
    fn from_be_byte_array() {
        let n = UIntEx::from_be_byte_array(&hex!("00112233445566778899aabbccddeeff").into());
        assert_eq!(&n.limbs, &[0x8899aabbccddeeff, 0x0011223344556677]);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "32")]
    fn from_le_byte_array() {
        let n = UIntEx::from_le_byte_array(&hex!("7766554433221100").into());
        assert_eq!(&n.limbs, &[0x44556677, 0x00112233]);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "64")]
    fn from_le_byte_array() {
        let n = UIntEx::from_le_byte_array(&hex!("ffeeddccbbaa99887766554433221100").into());
        assert_eq!(&n.limbs, &[0x8899aabbccddeeff, 0x0011223344556677]);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "32")]
    fn to_be_byte_array() {
        let expected_bytes = ByteArray::from(hex!("0011223344556677"));
        let actual_bytes = UIntEx::from_be_byte_array(&expected_bytes).to_be_byte_array();
        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "64")]
    fn to_be_byte_array() {
        let expected_bytes = ByteArray::from(hex!("00112233445566778899aabbccddeeff"));
        let actual_bytes = UIntEx::from_be_byte_array(&expected_bytes).to_be_byte_array();
        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "32")]
    fn to_le_byte_array() {
        let expected_bytes = ByteArray::from(hex!("7766554433221100"));
        let actual_bytes = UIntEx::from_le_byte_array(&expected_bytes).to_le_byte_array();
        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    #[cfg(feature = "generic-array")]
    #[cfg(target_pointer_width = "64")]
    fn to_le_byte_array() {
        let expected_bytes = ByteArray::from(hex!("ffeeddccbbaa99887766554433221100"));
        let actual_bytes = UIntEx::from_le_byte_array(&expected_bytes).to_le_byte_array();
        assert_eq!(expected_bytes, actual_bytes);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn from_be_hex() {
        let n = UIntEx::from_be_hex("0011223344556677");
        assert_eq!(&n.limbs, &[0x44556677, 0x00112233]);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn from_be_hex() {
        let n = UIntEx::from_be_hex("00112233445566778899aabbccddeeff");
        assert_eq!(&n.limbs, &[0x8899aabbccddeeff, 0x0011223344556677]);
    }

    #[test]
    #[cfg(target_pointer_width = "32")]
    fn from_le_hex() {
        let n = UIntEx::from_le_hex("7766554433221100");
        assert_eq!(&n.limbs, &[0x44556677, 0x00112233]);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn from_le_hex() {
        let n = UIntEx::from_le_hex("ffeeddccbbaa99887766554433221100");
        assert_eq!(&n.limbs, &[0x8899aabbccddeeff, 0x0011223344556677]);
    }

    #[test]
    fn is_zero() {
        assert!(bool::from(UIntEx::ZERO.is_zero()));
        assert!(!bool::from(UIntEx::ONE.is_zero()));
    }

    #[test]
    fn adc_no_carry() {
        let (res, carry) = UIntEx::ZERO.adc(&UIntEx::ONE, 0);
        assert_eq!(res, UIntEx::ONE);
        assert_eq!(carry, 0);
    }

    #[test]
    fn adc_with_carry() {
        let (res, carry) = UIntEx::MAX.adc(&UIntEx::ONE, 0);

        assert_eq!(res, UIntEx::ZERO);
        assert_eq!(carry, 1);
    }

    #[test]
    fn sbb_no_borrow() {
        let (res, borrow) = UIntEx::ONE.sbb(&UIntEx::ONE, 0);
        assert_eq!(res, UIntEx::ZERO);
        assert_eq!(borrow, 0);
    }

    #[test]
    fn sbb_with_borrow() {
        let (res, borrow) = UIntEx::ZERO.sbb(&UIntEx::ONE, 0);

        assert_eq!(res, UIntEx::MAX);
        assert_eq!(borrow, Limb::MAX);
    }

    #[test]
    fn wrapping_add_no_carry() {
        assert_eq!(UIntEx::ZERO.wrapping_add(&UIntEx::ONE), UIntEx::ONE);
    }

    #[test]
    fn wrapping_add_with_carry() {
        assert_eq!(UIntEx::MAX.wrapping_add(&UIntEx::ONE), UIntEx::ZERO);
    }

    #[test]
    fn checked_add_ok() {
        let result = UIntEx::ZERO.checked_add(&UIntEx::ONE);
        assert_eq!(result.unwrap(), UIntEx::ONE);
    }

    #[test]
    fn checked_add_overflow() {
        let result = UIntEx::MAX.checked_add(&UIntEx::ONE);
        assert!(!bool::from(result.is_some()));
    }

    #[test]
    fn wrapping_sub_no_borrow() {
        assert_eq!(UIntEx::ONE.wrapping_sub(&UIntEx::ONE), UIntEx::ZERO);
    }

    #[test]
    fn wrapping_sub_with_borrow() {
        assert_eq!(UIntEx::ZERO.wrapping_sub(&UIntEx::ONE), UIntEx::MAX);
    }

    #[test]
    fn checked_sub_ok() {
        let result = UIntEx::ONE.checked_sub(&UIntEx::ONE);
        assert_eq!(result.unwrap(), UIntEx::ZERO);
    }

    #[test]
    fn checked_sub_overflow() {
        let result = UIntEx::ZERO.checked_sub(&UIntEx::ONE);
        assert!(!bool::from(result.is_some()));
    }

    #[test]
    fn mul_wide_zero_and_one() {
        assert_eq!(
            UIntEx::ZERO.mul_wide(&UIntEx::ZERO),
            (UIntEx::ZERO, UIntEx::ZERO)
        );
        assert_eq!(
            UIntEx::ZERO.mul_wide(&UIntEx::ONE),
            (UIntEx::ZERO, UIntEx::ZERO)
        );
        assert_eq!(
            UIntEx::ONE.mul_wide(&UIntEx::ZERO),
            (UIntEx::ZERO, UIntEx::ZERO)
        );
        assert_eq!(
            UIntEx::ONE.mul_wide(&UIntEx::ONE),
            (UIntEx::ZERO, UIntEx::ONE)
        );
    }

    // TODO(tarcieri): add proptests for multiplication
    #[test]
    fn mul_wide_lo_only() {
        let primes: &[u32] = &[3, 5, 17, 256, 65537];

        for &a_int in primes {
            for &b_int in primes {
                let (hi, lo) = U64::from_u32(a_int).mul_wide(&U64::from_u32(b_int));
                let expected = U64::from_u64(a_int as u64 * b_int as u64);
                assert_eq!(lo, expected);
                assert!(bool::from(hi.is_zero()));
            }
        }
    }

    #[test]
    fn square() {
        let n = U64::from_u64(0xffff_ffff_ffff_ffff);
        let (hi, lo) = n.square().split();
        assert_eq!(lo, U64::from_u64(1));
        assert_eq!(hi, U64::from_u64(0xffff_ffff_ffff_fffe));
    }

    #[test]
    fn checked_mul_ok() {
        let n = U64::from_u32(0xffff_ffff);
        assert_eq!(
            n.checked_mul(&n).unwrap(),
            U64::from_u64(0xffff_fffe_0000_0001)
        );
    }

    #[test]
    fn checked_mul_overflow() {
        let n = U64::from_u64(0xffff_ffff_ffff_ffff);
        assert!(bool::from(n.checked_mul(&n).is_none()));
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
