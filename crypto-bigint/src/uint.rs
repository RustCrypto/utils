//! Big unsigned integers.

#![allow(clippy::needless_range_loop)]

use crate::{ops, Limb, NumBits, NumBytes, LIMB_BYTES};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// Constant panicking assertion.
// TODO(tarcieri): use const panic when stable.
// See: https://github.com/rust-lang/rust/issues/51999
macro_rules! const_assert {
    ($bool:expr, $msg:expr) => {
        [$msg][!$bool as usize]
    };
}

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

        let mut limbs = [0; LIMBS];
        let mut i = 0;
        let mut offset = LIMB_BYTES * LIMBS;

        while i < LIMBS {
            offset -= LIMB_BYTES;
            let mut j = 0;

            while j < LIMB_BYTES {
                limbs[i] = (limbs[i] << 8) | bytes[offset + j] as Limb;
                j += 1;
            }

            i += 1;
        }

        Self { limbs }
    }

    /// Create a new [`UInt`] from the provided little endian bytes.
    pub const fn from_le_bytes(bytes: &[u8]) -> Self {
        const_assert!(
            bytes.len() == LIMB_BYTES * LIMBS,
            "bytes are not the expected size"
        );

        let mut limbs = [0; LIMBS];
        let mut i = 0;

        while i < LIMBS {
            let mut j = LIMB_BYTES;
            let offset = i * LIMB_BYTES;

            while j > 0 {
                limbs[i] = (limbs[i] << 8) | bytes[offset + j - 1] as Limb;
                j -= 1;
            }

            i += 1;
        }

        Self { limbs }
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
    (U1088, 1088, "1088-bit"),
    (U1152, 1152, "1152-bit"),
    (U1216, 1216, "1216-bit"),
    (U1280, 1280, "1280-bit"),
    (U1344, 1344, "1344-bit"),
    (U1408, 1408, "1408-bit"),
    (U1472, 1472, "1472-bit"),
    (U1536, 1536, "1536-bit"),
    (U1600, 1600, "1600-bit"),
    (U1664, 1664, "1664-bit"),
    (U1728, 1728, "1728-bit"),
    (U1792, 1792, "1792-bit"),
    (U1856, 1856, "1856-bit"),
    (U1920, 1920, "1920-bit"),
    (U1984, 1984, "1984-bit"),
    (U2048, 2048, "2048-bit"),
    (U2112, 2112, "2112-bit"),
    (U2176, 2176, "2176-bit"),
    (U2240, 2240, "2240-bit"),
    (U2304, 2304, "2304-bit"),
    (U2368, 2368, "2368-bit"),
    (U2432, 2432, "2432-bit"),
    (U2496, 2496, "2496-bit"),
    (U2560, 2560, "2560-bit"),
    (U2624, 2624, "2624-bit"),
    (U2688, 2688, "2688-bit"),
    (U2752, 2752, "2752-bit"),
    (U2816, 2816, "2816-bit"),
    (U2880, 2880, "2880-bit"),
    (U2944, 2944, "2944-bit"),
    (U3008, 3008, "3008-bit"),
    (U3072, 3072, "3072-bit"),
    (U3136, 3136, "3136-bit"),
    (U3200, 3200, "3200-bit"),
    (U3264, 3264, "3264-bit"),
    (U3328, 3328, "3328-bit"),
    (U3392, 3392, "3392-bit"),
    (U3456, 3456, "3456-bit"),
    (U3520, 3520, "3520-bit"),
    (U3584, 3584, "3584-bit"),
    (U3648, 3648, "3648-bit"),
    (U3712, 3712, "3712-bit"),
    (U3776, 3776, "3776-bit"),
    (U3840, 3840, "3840-bit"),
    (U3904, 3904, "3904-bit"),
    (U3968, 3968, "3968-bit"),
    (U4032, 4032, "4032-bit"),
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

    // Maximum value for `UIntEx`
    #[cfg(target_pointer_width = "32")]
    const MAX_UINT_HEX: [u8; LIMB_BYTES * 2] = hex!("FFFFFFFFFFFFFFFF");
    #[cfg(target_pointer_width = "64")]
    const MAX_UINT_HEX: [u8; LIMB_BYTES * 2] = hex!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");

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
