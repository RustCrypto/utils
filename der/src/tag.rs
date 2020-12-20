//! ASN.1 tags.

use crate::{Decodable, Decoder, Encodable, Encoder, Error, ErrorKind, Length, Result};
use core::{convert::TryFrom, fmt};

/// ASN.1 tags.
#[derive(Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Tag {
    /// `BOOLEAN` tag.
    Boolean = 0x01,

    /// `INTEGER` tag.
    Integer = 0x02,

    /// `BIT STRING` tag.
    BitString = 0x03,

    /// `OCTET STRING` tag.
    OctetString = 0x04,

    /// `NULL` tag.
    Null = 0x05,

    /// `OBJECT IDENTIFIER` tag.
    ObjectIdentifier = 0x06,

    /// `SEQUENCE` tag.
    Sequence = 0x30,
}

impl TryFrom<u8> for Tag {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Tag> {
        match byte {
            0x01 => Ok(Tag::Boolean),
            0x02 => Ok(Tag::Integer),
            0x03 => Ok(Tag::BitString),
            0x04 => Ok(Tag::OctetString),
            0x05 => Ok(Tag::Null),
            0x06 => Ok(Tag::ObjectIdentifier),
            0x30 => Ok(Tag::Sequence),
            _ => Err(ErrorKind::UnknownTag { byte }.into()),
        }
    }
}

impl Tag {
    /// Assert that this [`Tag`] matches the provided expected tag.
    ///
    /// On mismatch, returns an [`Error`] with [`ErrorKind::UnexpectedTag`].
    pub fn assert_eq(self, expected: Tag) -> Result<Tag> {
        if self == expected {
            Ok(self)
        } else {
            Err(ErrorKind::UnexpectedTag {
                expected: Some(expected),
                actual: self,
            }
            .into())
        }
    }

    /// Names of ASN.1 type which corresponds to a given [`Tag`].
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Boolean => "BOOLEAN",
            Self::Integer => "INTEGER",
            Self::BitString => "BIT STRING",
            Self::OctetString => "OCTET STRING",
            Self::Null => "NULL",
            Self::ObjectIdentifier => "OBJECT IDENTIFIER",
            Self::Sequence => "SEQUENCE",
        }
    }
}

impl Decodable<'_> for Tag {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self> {
        decoder.byte().and_then(Self::try_from)
    }
}

impl Encodable for Tag {
    fn encoded_len(&self) -> Result<Length> {
        Ok(1u8.into())
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        encoder.byte(*self as u8)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.type_name())
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tag(0x{:02x}: {})", *self as u8, self.type_name())
    }
}
