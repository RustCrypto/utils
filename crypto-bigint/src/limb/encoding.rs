//! Limb encoding

use super::{Inner, Limb};
use crate::Encoding;

impl Encoding for Limb {
    // 32-bit
    #[cfg(target_pointer_width = "32")]
    const BIT_SIZE: usize = 32;
    #[cfg(target_pointer_width = "32")]
    const BYTE_SIZE: usize = 4;
    #[cfg(target_pointer_width = "32")]
    type Repr = [u8; 4];

    // 64-bit
    #[cfg(target_pointer_width = "64")]
    const BIT_SIZE: usize = 64;
    #[cfg(target_pointer_width = "64")]
    const BYTE_SIZE: usize = 8;
    #[cfg(target_pointer_width = "64")]
    type Repr = [u8; 8];

    #[inline]
    fn from_be_bytes(bytes: Self::Repr) -> Self {
        Limb(Inner::from_be_bytes(bytes))
    }

    #[inline]
    fn from_le_bytes(bytes: Self::Repr) -> Self {
        Limb(Inner::from_le_bytes(bytes))
    }

    #[inline]
    fn to_be_bytes(&self) -> Self::Repr {
        self.0.to_be_bytes()
    }

    #[inline]
    fn to_le_bytes(&self) -> Self::Repr {
        self.0.to_le_bytes()
    }
}
