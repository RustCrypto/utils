//! ASN.1 `ANY` type

use crate::{Decodable, Decoder, Error, Header, Length, Result, Tag};

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
}

impl<'a> Decodable<'a> for Any<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Any<'a>> {
        let header = Header::decode(decoder)?;
        let tag = header.tag;
        let len = usize::from(header.length);

        if len > decoder.len() {
            return Err(Error::Length { tag });
        }

        let (value, rest) = decoder.split_at(len);
        *decoder = rest;

        Self::new(tag, value)
    }
}
