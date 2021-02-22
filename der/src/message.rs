//! The [`Message`] pattern provided by this crate simplifies writing ASN.1 DER
//! decoders and encoders which map ASN.1 `SEQUENCE` types to Rust structs.

use crate::{asn1::sequence, Decodable, Encodable, Encoder, Length, Result, Tag, Tagged};

/// Messages encoded as an ASN.1 `SEQUENCE`.
///
/// The "message" pattern this trait provides is not an ASN.1 concept,
/// but rather a pattern for writing ASN.1 DER decoders and encoders which
/// map ASN.1 `SEQUENCE` types to Rust structs with a minimum of code.
///
/// Types which impl this trait receive blanket impls for the [`Decodable`],
/// [`Encodable`], and [`Tagged`] traits.
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
