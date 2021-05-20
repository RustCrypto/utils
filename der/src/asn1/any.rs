//! ASN.1 `ANY` type.

use crate::{
    BitString, ByteSlice, Choice, Decodable, Decoder, Encodable, Encoder, Error, ErrorKind,
    GeneralizedTime, Header, Ia5String, Length, Null, OctetString, PrintableString, Result,
    Sequence, Tag, UtcTime, Utf8String,
};
use core::convert::{TryFrom, TryInto};

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// ASN.1 `ANY`: represents any explicitly tagged ASN.1 value.
///
/// Technically `ANY` hasn't been a recommended part of ASN.1 since the X.209
/// revision from 1988. It was deprecated and replaced by Information Object
/// Classes in X.680 in 1994, and X.690 no longer refers to it whatsoever.
///
/// Nevertheless, this crate defines an [`Any`] type as it remains a familiar
/// and useful concept which is still extensively used in things like
/// PKI-related RFCs.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Any<'a> {
    /// Tag representing the type of the encoded value
    pub(crate) tag: Tag,

    /// Inner value encoded as bytes
    pub(crate) value: ByteSlice<'a>,
}

impl<'a> Any<'a> {
    /// Create a new [`Any`] from the provided [`Tag`] and slice.
    pub fn new(tag: Tag, value: &'a [u8]) -> Result<Self> {
        Ok(Self {
            tag,
            value: ByteSlice::new(value).map_err(|_| ErrorKind::Length { tag })?,
        })
    }

    /// Get the tag for this [`Any`] type.
    pub fn tag(self) -> Tag {
        self.tag
    }

    /// Get the [`Length`] of this [`Any`] type's value.
    pub fn len(self) -> Length {
        self.value.len()
    }

    /// Is the body of this [`Any`] type empty?
    pub fn is_empty(self) -> bool {
        self.value.is_empty()
    }

    /// Is this value an ASN.1 NULL value?
    pub fn is_null(self) -> bool {
        Null::try_from(self).is_ok()
    }

    /// Get the raw value for this [`Any`] type as a byte slice.
    pub fn as_bytes(self) -> &'a [u8] {
        self.value.as_bytes()
    }

    /// Attempt to decode an ASN.1 `BIT STRING`.
    pub fn bit_string(self) -> Result<BitString<'a>> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `CONTEXT-SPECIFIC`, creating a new
    /// nested [`Decoder`] and calling the provided argument with it.
    ///
    /// The provided `tag` should contain the lower 6-bits of the context
    /// specific tag, sans the leading `10` high bits.
    pub fn context_specific<F, T>(self, context_specific_tag: u8, f: F) -> Result<T>
    where
        F: FnOnce(&mut Decoder<'a>) -> Result<T>,
    {
        let tag = Tag::context_specific(context_specific_tag)?;

        if self.tag != tag {
            return Err(ErrorKind::UnexpectedTag {
                expected: Some(tag),
                actual: self.tag,
            }
            .into());
        }

        let mut decoder = Decoder::new(self.as_bytes());
        let result = f(&mut decoder)?;
        decoder.finish(result)
    }

    /// Attempt to decode an `OPTIONAL` ASN.1 `CONTEXT-SPECIFIC` field.
    ///
    /// Returns `Ok(None)` if the tag is not of the expected type.
    pub fn context_specific_optional<F, T>(
        self,
        context_specific_tag: u8,
        f: F,
    ) -> Result<Option<T>>
    where
        F: FnOnce(&mut Decoder<'a>) -> Result<T>,
    {
        if self.tag == Tag::context_specific(context_specific_tag)? {
            self.context_specific(context_specific_tag, f).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Attempt to decode an ASN.1 `GeneralizedTime`.
    pub fn generalized_time(self) -> Result<GeneralizedTime> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `IA5String`.
    pub fn ia5_string(self) -> Result<Ia5String<'a>> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `OCTET STRING`.
    pub fn octet_string(self) -> Result<OctetString<'a>> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `OBJECT IDENTIFIER`.
    #[cfg(feature = "oid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
    pub fn oid(self) -> Result<ObjectIdentifier> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `PrintableString`.
    pub fn printable_string(self) -> Result<PrintableString<'a>> {
        self.try_into()
    }

    /// Attempt to decode this value an ASN.1 `SEQUENCE`, creating a new
    /// nested [`Decoder`] and calling the provided argument with it.
    pub fn sequence<F, T>(self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Decoder<'a>) -> Result<T>,
    {
        Sequence::try_from(self)?.decode_nested(f)
    }

    /// Attempt to decode an ASN.1 `UTCTime`.
    pub fn utc_time(self) -> Result<UtcTime> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `UTF8String`.
    pub fn utf8_string(self) -> Result<Utf8String<'a>> {
        self.try_into()
    }
}

impl<'a> Choice<'a> for Any<'a> {
    fn can_decode(_: Tag) -> bool {
        true
    }
}

impl<'a> Decodable<'a> for Any<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Any<'a>> {
        let header = Header::decode(decoder)?;
        let tag = header.tag;
        let value = decoder
            .bytes(header.length)
            .map_err(|_| ErrorKind::Length { tag })?;
        Self::new(tag, value)
    }
}

impl<'a> Encodable for Any<'a> {
    fn encoded_len(&self) -> Result<Length> {
        self.len().for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Header::new(self.tag, self.len())?.encode(encoder)?;
        encoder.bytes(self.as_bytes())
    }
}

impl<'a> TryFrom<&'a [u8]> for Any<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Any<'a>> {
        Any::from_der(bytes)
    }
}
