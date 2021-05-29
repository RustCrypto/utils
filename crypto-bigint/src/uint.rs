//! Big unsigned integers.

#![allow(clippy::needless_range_loop)]

mod decoder;

use self::decoder::Decoder;
use crate::{ops, Limb, NumBits, NumBytes, LIMB_BYTES};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// Big unsigned integer.
///
/// Generic over the given number of `LIMBS`
// TODO(tarcieri): make generic around a specified number of bits.
#[derive(Copy, Clone, Debug)]
pub struct UInt<const LIMBS: usize> {
    /// Inner limb array.
    ///
    /// Stored from least significant to most significant.
    limbs: [Limb; LIMBS],
}

impl<const LIMBS: usize> UInt<LIMBS> {
    /// The value `0`.
    pub const ZERO: Self = Self::from_u8(0);

    /// The value `1`.
    pub const ONE: Self = Self::from_u8(1);

    /// Create a [`UInt`] from a `u8` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u8>`
    pub const fn from_u8(n: u8) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u16` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u16>`
    pub const fn from_u16(n: u16) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u32` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u32>`
    pub const fn from_u32(n: u32) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u64` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u64>`
    #[cfg(target_pointer_width = "32")]
    pub const fn from_u64(n: u64) -> Self {
        const_assert!(LIMBS >= 2, "number of limbs must be two or greater");
        let mut limbs = [0; LIMBS];
        limbs[0] = (n & 0xFFFF) as u32;
        limbs[1] = (n >> 32) as u32;
        Self { limbs }
    }

    /// Create a [`UInt`] from a `u64` (const-friendly)
    // TODO(tarcieri): replace with `const impl From<u64>`
    #[cfg(target_pointer_width = "64")]
    pub const fn from_u64(n: u64) -> Self {
        const_assert!(LIMBS >= 1, "number of limbs must be greater than zero");
        let mut limbs = [0; LIMBS];
        limbs[0] = n as Limb;
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

    /// Determine if this [`UInt`] is equal to zero.
    ///
    /// # Returns
    ///
    /// If zero, return `Choice(1)`.  Otherwise, return `Choice(0)`.
    pub fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }

    /// Computes `a + b + carry`, returning the result along with the new carry.
    /// 64-bit version.
    #[inline(always)]
    pub const fn adc(&self, rhs: &Self, mut carry: Limb) -> (Self, Limb) {
        let mut limbs = [0; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let (w, c) = ops::adc(self.limbs[i], rhs.limbs[i], carry);
            limbs[i] = w;
            carry = c;
            i += 1;
        }

        (Self { limbs }, carry)
    }

    /// Perform checked addition, returning [`CtOption`] only if the operation
    /// did not overflow.
    pub fn checked_add(&self, rhs: &Self) -> CtOption<Self> {
        let (result, carry) = self.adc(rhs, 0);
        CtOption::new(result, !Choice::from(carry as u8))
    }

    /// Borrow the limbs of this [`UInt`].
    pub fn limbs(&self) -> &[Limb; LIMBS] {
        &self.limbs
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
        debug_assert!(LIMBS > 0, "limbs must be non-zero");
        Self::from_u64(n)
    }
}

impl<const LIMBS: usize> PartialEq for UInt<LIMBS> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl<const LIMBS: usize> Eq for UInt<LIMBS> {}

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

macro_rules! impl_biguint_aliases {
    ($(($name:ident, $bits:expr, $doc:expr)),+) => {
        $(
            #[doc = $doc]
            #[doc="unsigned big integer"]
            #[cfg(target_pointer_width = "32")]
            pub type $name = UInt<{$bits / 32}>;

            #[doc = $doc]
            #[doc="unsigned big integer"]
            #[cfg(target_pointer_width = "64")]
            pub type $name = UInt<{$bits / 64}>;

            impl NumBits for $name {
                const NUM_BITS: usize = $bits;
            }

            impl NumBytes for $name {
                const NUM_BYTES: usize = $bits / 8;
            }
        )+
     };
}

// TODO(tarcieri): make generic around a specified number of bits.
impl_biguint_aliases! {
    (U64, 64, "64-bit"),
    (U128, 128, "128-bit"),
    (U192, 192, "192-bit"),
    (U256, 256, "256-bit"),
    (U320, 320, "320-bit"),
    (U384, 384, "384-bit"),
    (U448, 448, "448-bit"),
    (U512, 512, "512-bit"),
    (U576, 576, "576-bit"),
    (U640, 640, "640-bit"),
    (U704, 704, "704-bit"),
    (U768, 768, "768-bit"),
    (U832, 832, "832-bit"),
    (U896, 896, "896-bit"),
    (U960, 960, "960-bit"),
    (U1024, 1024, "1024-bit"),
    (U2048, 2048, "2048-bit"),
    (U4096, 4096, "4096-bit")
}

#[cfg(test)]
mod tests {
    use crate::LIMB_BYTES;
    use hex_literal::hex;

    // 2-limb example that's twice as wide as the native word size
    #[cfg(target_pointer_width = "64")]
    use super::U128 as UIntEx;
    #[cfg(target_pointer_width = "32")]
    use super::U64 as UIntEx;

    /// Maximum value for `UIntEx`
    #[cfg(target_pointer_width = "32")]
    const MAX_UINT_HEX: [u8; LIMB_BYTES * 2] = hex!("FFFFFFFFFFFFFFFF");
    #[cfg(target_pointer_width = "64")]
    const MAX_UINT_HEX: [u8; LIMB_BYTES * 2] = hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

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
    fn adc_carry_out() {
        let n = UIntEx::from_be_bytes(&MAX_UINT_HEX);
        let (res, carry) = n.adc(&UIntEx::ONE, 0);

        assert_eq!(res, UIntEx::ZERO);
        assert_eq!(carry, 1);
    }

    #[test]
    fn checked_add_ok() {
        let result = UIntEx::ZERO.checked_add(&UIntEx::ONE);
        assert_eq!(result.unwrap(), UIntEx::ONE);
    }

    #[test]
    fn checked_add_overflow() {
        let n = UIntEx::from_be_bytes(&MAX_UINT_HEX);
        let result = n.checked_add(&UIntEx::ONE);
        assert!(!bool::from(result.is_some()));
    }
}
