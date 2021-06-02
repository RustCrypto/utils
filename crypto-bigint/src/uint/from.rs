//! `From`-like conversions for [`UInt`].

use crate::{Limb, Split, UInt, LIMB_BYTES, U128, U64};

impl<const LIMBS: usize> UInt<LIMBS> {
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

        let lo = U64::from_u64((n & 0xffff_ffff_ffff_ffff) as u64);
        let hi = U64::from_u64((n >> 64) as u64);

        let mut limbs = [0; LIMBS];

        let mut i = 0;
        while i < lo.limbs.len() {
            limbs[i] = lo.limbs[i];
            i += 1;
        }

        let mut j = 0;
        while j < hi.limbs.len() {
            limbs[i + j] = hi.limbs[j];
            j += 1;
        }

        Self { limbs }
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
        debug_assert!(LIMBS >= (8 / LIMB_BYTES), "not enough limbs");
        Self::from_u64(n)
    }
}

impl<const LIMBS: usize> From<u128> for UInt<LIMBS> {
    fn from(n: u128) -> Self {
        // TODO(tarcieri): const where clause when possible
        debug_assert!(LIMBS >= (16 / LIMB_BYTES), "not enough limbs");
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

#[cfg(test)]
mod tests {
    use crate::U128;

    #[cfg(target_pointer_width = "32")]
    use crate::U64 as UIntEx;

    #[cfg(target_pointer_width = "64")]
    use crate::U128 as UIntEx;

    #[test]
    fn from_u8() {
        let n = UIntEx::from(42u8);
        assert_eq!(n.limbs(), &[42, 0]);
    }

    #[test]
    fn from_u16() {
        let n = UIntEx::from(42u16);
        assert_eq!(n.limbs(), &[42, 0]);
    }

    #[test]
    fn from_u64() {
        let n = UIntEx::from(42u64);
        assert_eq!(n.limbs(), &[42, 0]);
    }

    #[test]
    fn from_u128() {
        let n = U128::from(42u128);
        assert_eq!(&n.limbs()[..2], &[42, 0]);
        assert_eq!(u128::from(n), 42u128);
    }
}
