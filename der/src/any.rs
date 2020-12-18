//! ASN.1 `ANY` type

use crate::{
    BitString, Decodable, Decoder, Error, Header, Integer, Length, Null, OctetString, Result,
    Sequence, Tag,
};
use core::convert::{TryFrom, TryInto};

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// ASN.1 `ANY` type: represents any ASN.1 value
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Any<'a> {
    /// Tag representing the type of the encoded value
    tag: Tag,

    /// Inner value encoded as bytes
    value: &'a [u8],
}

impl<'a> Any<'a> {
    /// Create a new [`Any`] from the provided tag and slice
    pub fn new(tag: Tag, value: &'a [u8]) -> Result<Self> {
        if value.len() <= Length::max() {
            Ok(Self { tag, value })
        } else {
            Err(Error::Length { tag })
        }
    }

    /// Get the tag for this [`Any`] type
    pub fn tag(self) -> Tag {
        self.tag
    }

    /// Get the value for this [`Any`] type as a byte slice
    pub fn value(self) -> &'a [u8] {
        self.value
    }

    /// Attempt to decode an ASN.1 `BIT STRING`
    pub fn bit_string(self) -> Result<BitString<'a>> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `INTEGER`
    pub fn integer(self) -> Result<Integer> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `NULL` value
    pub fn null(self) -> Result<Null> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `OCTET STRING`
    pub fn octet_string(self) -> Result<OctetString<'a>> {
        self.try_into()
    }

    /// Attempt to decode an ASN.1 `OBJECT IDENTIFIER`
    #[cfg(feature = "oid")]
    #[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
    pub fn oid(self) -> Result<ObjectIdentifier> {
        self.try_into()
    }

    /// Attempt to decode this value an ASN.1 `SEQUENCE`, creating a new
    /// nested [`Decoder`] and calling the provided argument with it.
    pub fn sequence<F, T>(self, f: F) -> Result<T>
    where
        F: FnOnce(Decoder<'a>) -> Result<T>,
    {
        Sequence::try_from(self).and_then(|seq| f(seq.decoder()))
    }
}

impl<'a> Decodable<'a> for Any<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Any<'a>> {
        let header = Header::decode(decoder)?;
        let tag = header.tag;
        let len = usize::from(header.length);
        let value = decoder.bytes(len).map_err(|_| Error::Length { tag })?;
        Self::new(tag, value)
    }
}
