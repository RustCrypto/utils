//! ASN.1 `SEQUENCE` support.

use crate::{
    Any, ByteSlice, Decoder, Encodable, Encoder, Error, Header, Length, Result, Tag, Tagged,
};
use core::convert::TryFrom;

/// Obtain the length of an ASN.1 `SEQUENCE` of [`Encodable`] values when
/// serialized as ASN.1 DER, including the `SEQUENCE` tag and length prefix.
pub(crate) fn encoded_len(encodables: &[&dyn Encodable]) -> Result<Length> {
    let inner_len = encoded_len_inner(encodables)?;
    Header::new(Tag::Sequence, inner_len)?.encoded_len() + inner_len
}

/// Obtain the inner length of an ASN.1 `SEQUENCE` of [`Encodable`] values
/// excluding the tag and length.
pub(crate) fn encoded_len_inner(encodables: &[&dyn Encodable]) -> Result<Length> {
    encodables
        .iter()
        .fold(Ok(Length::zero()), |sum, encodable| {
            sum + encodable.encoded_len()?
        })
}

/// ASN.1 `SEQUENCE` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Sequence<'a> {
    /// Inner value
    inner: ByteSlice<'a>,
}

impl<'a> Sequence<'a> {
    /// Create a new [`Sequence`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        ByteSlice::new(slice)
            .map(|inner| Self { inner })
            .map_err(|_| Error::Length { tag: Self::TAG })
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    /// Obtain a [`Decoder`] for the data in this [`Sequence`]
    pub fn decoder(&self) -> Decoder<'a> {
        Decoder::new(self.as_bytes())
    }
}

impl AsRef<[u8]> for Sequence<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> TryFrom<Any<'a>> for Sequence<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Sequence<'a>> {
        any.tag().assert_eq(Tag::Sequence)?;
        Self::new(any.as_bytes())
    }
}

impl<'a> From<Sequence<'a>> for Any<'a> {
    fn from(seq: Sequence<'a>) -> Any<'a> {
        Any {
            tag: Tag::Sequence,
            value: seq.inner,
        }
    }
}

impl<'a> Encodable for Sequence<'a> {
    fn encoded_len(&self) -> Result<Length> {
        Any::from(*self).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(*self).encode(encoder)
    }
}

impl<'a> Tagged for Sequence<'a> {
    const TAG: Tag = Tag::Sequence;
}
