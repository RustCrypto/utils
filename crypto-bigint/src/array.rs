//! Interop support for `generic-array`

use crate::uint::*;
use generic_array::{typenum, ArrayLength, GenericArray};

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

    /// Serialize to a big-endian byte array.
    fn to_be_byte_array(&self) -> ByteArray<Self>;

    /// Serialize to a little-endian byte array.
    fn to_le_byte_array(&self) -> ByteArray<Self>;
}

macro_rules! impl_biguint_array_encoding {
    ($(($uint:ident, $bytes:path)),+) => {
        $(
            #[cfg_attr(docsrs, doc(cfg(feature = "generic-array")))]
            impl ArrayEncoding for $uint {
                type ByteSize = $bytes;

                #[inline]
                fn from_be_byte_array(bytes: &ByteArray<Self>) -> Self {
                    Self::from_be_bytes(bytes)
                }

                #[inline]
                fn from_le_byte_array(bytes: &ByteArray<Self>) -> Self {
                    Self::from_le_bytes(bytes)
                }

                #[inline]
                fn to_be_byte_array(&self) -> ByteArray<Self> {
                    let mut result = GenericArray::default();
                    self.to_be_bytes(&mut result);
                    result
                }

                #[inline]
                fn to_le_byte_array(&self) -> ByteArray<Self> {
                    let mut result = GenericArray::default();
                    self.to_le_bytes(&mut result);
                    result
                }
            }
        )+
     };
}

impl_biguint_array_encoding! {
    (U64, typenum::U8),
    (U128, typenum::U16),
    (U192, typenum::U24),
    (U256, typenum::U32),
    (U384, typenum::U48),
    (U448, typenum::U56),
    (U512, typenum::U64),
    (U768, typenum::U96),
    (U896, typenum::U112),
    (U1024, typenum::U128),
    (U1536, typenum::U192),
    (U1792, typenum::U224),
    (U2048, typenum::U256),
    (U3072, typenum::U384),
    (U3584, typenum::U448),
    (U4096, typenum::U512),
    (U6144, typenum::U768),
    (U8192, typenum::U1024)
}
