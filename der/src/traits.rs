//! Trait definitions

use crate::{asn1::sequence, Any, Decoder, Encoder, Error, Length, Result, Tag};
use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use {
    crate::ErrorKind,
    alloc::vec::Vec,
    core::{convert::TryInto, iter},
};

/// Decoding trait.
pub trait Decodable<'a>: Sized {
    /// Attempt to decode this message using the provided decoder.
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self>;
}

impl<'a, T> Decodable<'a> for T
where
    T: TryFrom<Any<'a>, Error = Error>,
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<T> {
        Any::decode(decoder)
            .and_then(Self::try_from)
            .or_else(|e| decoder.error(e.kind()))
    }
}

/// Encoding trait.
pub trait Encodable {
    /// Compute the length of this value in bytes when encoded as ASN.1 DER.
    fn encoded_len(&self) -> Result<Length>;

    /// Encode this value as ASN.1 DER using the provided [`Encoder`].
    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()>;

    /// Encode this message as ASN.1 DER, appending it to the provided
    /// byte vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn encode_to_vec(&self, buf: &mut Vec<u8>) -> Result<Length> {
        let expected_len = usize::from(self.encoded_len()?);
        buf.reserve(expected_len);
        buf.extend(iter::repeat(0).take(expected_len));

        let mut encoder = Encoder::new(buf);
        self.encode(&mut encoder)?;
        let actual_len = encoder.finish()?.len();

        if expected_len != actual_len {
            return Err(ErrorKind::Underlength {
                expected: expected_len.try_into()?,
                actual: actual_len.try_into()?,
            }
            .into());
        }

        actual_len.try_into()
    }

    /// Serialize this message as a byte vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn to_vec(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.encode_to_vec(&mut buf)?;
        Ok(buf)
    }
}

/// Types with an associated ASN.1 [`Tag`].
pub trait Tagged {
    /// ASN.1 tag
    const TAG: Tag;
}

/// Messages encoded as an ASN.1 `SEQUENCE`.
///
/// This wraps up a common pattern for ASN.1 encoding.
///
/// Types which impl this trait receive blanket impls for the [`Decodable`],
/// [`Encodable`], and [`Tagged`] traits.
// TODO(tarcieri): ensure all `Message` types impl `Decodable`
pub trait Message<'a>: Decodable<'a> {
    /// Call the provided function with a slice of [`Encodable`] trait objects
    /// representing the fields of this message.
    ///
    /// This method uses a callback because structs with fields which aren't
    /// directly [`Encodable`] may need to construct temporary values from
    /// their fields prior to encoding.
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>;
}

impl<'a, M> Encodable for M
where
    M: Message<'a>,
{
    fn encoded_len(&self) -> Result<Length> {
        self.fields(sequence::encoded_len)
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.fields(|fields| encoder.sequence(fields))
    }
}

impl<'a, M> Tagged for M
where
    M: Message<'a>,
{
    const TAG: Tag = Tag::Sequence;
}
