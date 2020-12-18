//! ASN.1 tags.

use crate::{Decodable, Decoder, Error, Result};
use core::convert::TryFrom;

/// ASN.1 tags.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Tag {
    /// ASN.1 `INTEGER` tag
    Integer = 0x02,

    /// ASN.1 `BIT STRING` tag
    BitString = 0x03,

    /// ASN.1 `OCTET STRING` tag
    OctetString = 0x04,

    /// ASN.1 `NULL` tag
    Null = 0x05,

    /// ASN.1 `OBJECT IDENTIFIER` tag
    ObjectIdentifier = 0x06,

    /// ASN.1 `SEQUENCE` tag
    Sequence = 0x30,
}

impl Tag {
    /// Expect a specific tag type, returning an error if the tag is not the
    /// one we're expecting
    pub fn expect(self, expected: Tag) -> Result<Tag> {
        if self == expected {
            Ok(self)
        } else {
            Err(Error::UnexpectedTag {
                expected: Some(expected),
                actual: self,
            })
        }
    }
}

impl Decodable<'_> for Tag {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        // TODO(tarcieri): move `decode::byte` to `Decoder`
        crate::decoder::byte(decoder).and_then(Self::try_from)
    }
}

impl TryFrom<u8> for Tag {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Tag> {
        match byte {
            0x02 => Ok(Tag::Integer),
            0x03 => Ok(Tag::BitString),
            0x04 => Ok(Tag::OctetString),
            0x05 => Ok(Tag::Null),
            0x06 => Ok(Tag::ObjectIdentifier),
            0x30 => Ok(Tag::Sequence),
            _ => Err(Error::UnknownTag { byte }),
        }
    }
}
