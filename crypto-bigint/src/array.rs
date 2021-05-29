//! Interop support for `generic-array`

use crate::uint::*;
use generic_array::{ArrayLength, GenericArray};

/// Alias for a byte array whose size is defined by [`ArrayEncoding::ByteSize`].
#[cfg_attr(docsrs, doc(cfg(feature = "generic-array")))]
pub type ByteArray<T> = GenericArray<u8, <T as ArrayEncoding>::ByteSize>;

/// Support for encoding a big integer as a `GenericArray`.
#[cfg_attr(docsrs, doc(cfg(feature = "generic-array")))]
pub trait ArrayEncoding: Sized {
    /// Size of a byte array which encodes a big integer.
    type ByteSize: ArrayLength<u8>;

    /// Deserialize from a big-endian byte array.
    fn from_be_byte_array(bytes: &ByteArray<Self>) -> Self;

    /// Deserialize from a little-endian byte array.
    fn from_le_byte_array(bytes: &ByteArray<Self>) -> Self;
}

macro_rules! impl_biguint_array_encoding {
    ($(($uint:ident, $bytes:ident)),+) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(feature = "generic-array")))]
            impl ArrayEncoding for $uint {
                type ByteSize = crate::consts::$bytes;

                #[inline]
                fn from_be_byte_array(bytes: &ByteArray<Self>) -> Self {
                    Self::from_be_bytes(bytes)
                }

                #[inline]
                fn from_le_byte_array(bytes: &ByteArray<Self>) -> Self {
                    Self::from_le_bytes(bytes)
                }
            }
        )+
     };
}

impl_biguint_array_encoding! {
    (U64, U8),
    (U128, U16),
    (U192, U24),
    (U256, U32),
    (U320, U40),
    (U384, U48),
    (U448, U56),
    (U512, U64),
    (U576, U72),
    (U640, U80),
    (U704, U88),
    (U768, U96),
    (U832, U104),
    (U896, U112),
    (U960, U120),
    (U1024, U128),
    (U2048, U256),
    (U4096, U512)
}
