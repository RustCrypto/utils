//! ASN.1 `OCTET STRING` support.

use crate::{Any, ByteSlice, Encodable, Encoder, Error, ErrorKind, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `OCTET STRING` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OctetString<'a> {
    /// Inner value
    inner: ByteSlice<'a>,
}

impl<'a> OctetString<'a> {
    /// Create a new [`OctetString`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        ByteSlice::new(slice)
            .map(|inner| Self { inner })
            .map_err(|_| ErrorKind::Length { tag: Self::TAG }.into())
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }
}

impl AsRef<[u8]> for OctetString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&OctetString<'a>> for OctetString<'a> {
    fn from(value: &OctetString<'a>) -> OctetString<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for OctetString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<OctetString<'a>> {
        any.tag().assert_eq(Tag::OctetString)?;
        Self::new(any.as_bytes())
    }
}

impl<'a> From<OctetString<'a>> for Any<'a> {
    fn from(octet_string: OctetString<'a>) -> Any<'a> {
        Any {
            tag: Tag::OctetString,
            value: octet_string.inner,
        }
    }
}

impl<'a> From<OctetString<'a>> for &'a [u8] {
    fn from(octet_string: OctetString<'a>) -> &'a [u8] {
        octet_string.as_bytes()
    }
}

impl<'a> Encodable for OctetString<'a> {
    fn encoded_len(&self) -> Result<Length> {
        Any::from(*self).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(*self).encode(encoder)
    }
}

impl<'a> Tagged for OctetString<'a> {
    const TAG: Tag = Tag::OctetString;
}
