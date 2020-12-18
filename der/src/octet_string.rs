//! ASN.1 `OCTET STRING` support.

use crate::{length, Any, Encodable, Encoder, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `OCTET STRING` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OctetString<'a> {
    /// Inner value
    inner: &'a [u8],
}

impl<'a> OctetString<'a> {
    /// Create a new [`OctetString`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        if slice.len() <= Length::max() {
            Ok(Self { inner: slice })
        } else {
            Err(Error::Length {
                tag: Tag::OctetString,
            })
        }
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner
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
        any.tag().expect(Tag::OctetString)?;
        Self::new(any.as_bytes())
    }
}

impl<'a> From<OctetString<'a>> for Any<'a> {
    fn from(string: OctetString<'a>) -> Any<'a> {
        Any::new(Tag::OctetString, string.inner).expect(length::ERROR_MSG)
    }
}

impl<'a> From<OctetString<'a>> for &'a [u8] {
    fn from(string: OctetString<'a>) -> &'a [u8] {
        string.inner
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
