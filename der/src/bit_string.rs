//! ASN.1 `BIT STRING` support.

use crate::{length, Any, Encodable, Encoder, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `BIT STRING` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BitString<'a> {
    /// Inner value
    inner: &'a [u8],
}

impl<'a> BitString<'a> {
    /// Create a new [`BitString`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        if slice.len() <= Length::max() {
            Ok(Self { inner: slice })
        } else {
            Err(Error::Length {
                tag: Tag::BitString,
            })
        }
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner
    }
}

impl AsRef<[u8]> for BitString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> From<&BitString<'a>> for BitString<'a> {
    fn from(value: &BitString<'a>) -> BitString<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for BitString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BitString<'a>> {
        any.tag().expect(Tag::BitString)?;
        Self::new(any.as_bytes())
    }
}

impl<'a> From<BitString<'a>> for Any<'a> {
    fn from(bit_string: BitString<'a>) -> Any<'a> {
        Any::new(Tag::BitString, bit_string.inner).expect(length::ERROR_MSG)
    }
}

impl<'a> From<BitString<'a>> for &'a [u8] {
    fn from(string: BitString<'a>) -> &'a [u8] {
        string.inner
    }
}

impl<'a> Encodable for BitString<'a> {
    fn encoded_len(&self) -> Result<Length> {
        Any::from(*self).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(*self).encode(encoder)
    }
}

impl<'a> Tagged for BitString<'a> {
    const TAG: Tag = Tag::BitString;
}
