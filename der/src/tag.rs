//! ASN.1 tags.

use crate::{Decodable, Decoder, Encodable, Encoder, Error, ErrorKind, Length, Result};
use core::{convert::TryFrom, fmt};

/// Indicator bit for constructed form encoding (i.e. vs primitive form)
const CONSTRUCTED_FLAG: u8 = 0b100000;

/// Indicator bit for context-specific types
const CONTEXT_SPECIFIC_FLAG: u8 = 0b10000000;

/// Types with an associated ASN.1 [`Tag`].
pub trait Tagged {
    /// ASN.1 tag
    const TAG: Tag;
}

/// ASN.1 tags.
///
/// Tags are the leading byte of the Tag-Length-Value encoding used by ASN.1
/// DER and identify the type of the subsequent value.
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(clippy::identity_op)]
#[non_exhaustive]
#[repr(u8)]
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

    /// `UTF8String` tag.
    Utf8String = 0x0C,

    /// `SET` and `SET OF` tag.
    Set = 0x11,

    /// `PrintableString` tag.
    PrintableString = 0x13,

    /// `IA5String` tag.
    Ia5String = 0x16,

    /// `UTCTime` tag.
    UtcTime = 0x17,

    /// `GeneralizedTime` tag.
    GeneralizedTime = 0x18,

    /// `SEQUENCE` tag.
    ///
    /// Note that the universal tag number for `SEQUENCE` is technically `0x10`
    /// however we presently only support the constructed form, which has the
    /// 6th bit (i.e. `0x20`) set.
    Sequence = 0x10 | CONSTRUCTED_FLAG,

    /// Context-specific tag (0) unique to a particular structure.
    ContextSpecific0 = 0 | CONTEXT_SPECIFIC_FLAG | CONSTRUCTED_FLAG,

    /// Context-specific tag (1) unique to a particular structure.
    ContextSpecific1 = 1 | CONTEXT_SPECIFIC_FLAG | CONSTRUCTED_FLAG,

    /// Context-specific tag (2) unique to a particular structure.
    ContextSpecific2 = 2 | CONTEXT_SPECIFIC_FLAG | CONSTRUCTED_FLAG,

    /// Context-specific tag (3) unique to a particular structure.
    ContextSpecific3 = 3 | CONTEXT_SPECIFIC_FLAG | CONSTRUCTED_FLAG,
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
            0x0C => Ok(Tag::Utf8String),
            0x11 => Ok(Tag::Set),
            0x13 => Ok(Tag::PrintableString),
            0x16 => Ok(Tag::Ia5String),
            0x17 => Ok(Tag::UtcTime),
            0x18 => Ok(Tag::GeneralizedTime),
            0x30 => Ok(Tag::Sequence),
            0xA0 => Ok(Tag::ContextSpecific0),
            0xA1 => Ok(Tag::ContextSpecific1),
            0xA2 => Ok(Tag::ContextSpecific2),
            0xA3 => Ok(Tag::ContextSpecific3),
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
            Self::Utf8String => "UTF8String",
            Self::Set => "SET",
            Self::PrintableString => "PrintableString",
            Self::Ia5String => "IA5String",
            Self::UtcTime => "UTCTime",
            Self::GeneralizedTime => "GeneralizedTime",
            Self::Sequence => "SEQUENCE",
            Self::ContextSpecific0 => "Context Specific 0",
            Self::ContextSpecific1 => "Context Specific 1",
            Self::ContextSpecific2 => "Context Specific 2",
            Self::ContextSpecific3 => "Context Specific 3",
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
        Ok(Length::one())
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
