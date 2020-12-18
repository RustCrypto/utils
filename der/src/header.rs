//! ASN.1 DER headers

use crate::{Decodable, Decoder, Error, Length, Result, Tag};

/// ASN.1 DER headers: tag + length component of TLV-encoded values
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Header {
    /// Tag representing the type of the encoded value
    pub tag: Tag,

    /// Length of the encoded value
    pub length: Length,
}

impl Decodable<'_> for Header {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Header> {
        let tag = Tag::decode(decoder)?;

        let length = Length::decode(decoder).map_err(|e| {
            if e == Error::Overlength {
                Error::Length { tag }
            } else {
                e
            }
        })?;

        Ok(Self { tag, length })
    }
}
