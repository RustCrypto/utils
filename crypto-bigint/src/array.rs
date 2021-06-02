//! Interop support for `generic-array`

use core::ops::Add;
use generic_array::{typenum::Unsigned, ArrayLength, GenericArray};

/// Alias for a byte array whose size is defined by [`ArrayEncoding::ByteSize`].
#[cfg_attr(docsrs, doc(cfg(feature = "generic-array")))]
pub type ByteArray<T> = GenericArray<u8, <T as ArrayEncoding>::ByteSize>;

/// Support for encoding a big integer as a `GenericArray`.
#[cfg_attr(docsrs, doc(cfg(feature = "generic-array")))]
pub trait ArrayEncoding: Sized {
    /// Size of a byte array which encodes a big integer.
    type ByteSize: ArrayLength<u8> + Add + Eq + Ord + Unsigned;

    /// Deserialize from a big-endian byte array.
    fn from_be_byte_array(bytes: &ByteArray<Self>) -> Self;

    /// Deserialize from a little-endian byte array.
    fn from_le_byte_array(bytes: &ByteArray<Self>) -> Self;

    /// Serialize to a big-endian byte array.
    fn to_be_byte_array(&self) -> ByteArray<Self>;

    /// Serialize to a little-endian byte array.
    fn to_le_byte_array(&self) -> ByteArray<Self>;
}
