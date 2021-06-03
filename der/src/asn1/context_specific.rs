//! Context-specific field.

use crate::{
    asn1::Any, Choice, Decodable, Encodable, Encoder, Error, ErrorKind, Header, Length, Result,
    Tag, TagNumber,
};
use core::convert::TryFrom;

/// Context-specific field.
///
/// This type encodes a field which is specific to a particular context,
/// and is identified by a [`TagNumber`].
///
/// Any context-specific field can be decoded/encoded with this type.
/// The intended use is to dynamically dispatch off of the context-specific
/// tag number when decoding, which allows support for extensions, which are
/// denoted in an ASN.1 schema using the `...` ellipsis extension marker.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ContextSpecific<'a> {
    /// Context-specific tag number sans the leading `0b10000000` class
    /// identifier bit and `0b100000` constructed flag.
    pub tag_number: TagNumber,

    /// Value of the field.
    pub value: Any<'a>,
}

impl<'a> Choice<'a> for ContextSpecific<'a> {
    fn can_decode(tag: Tag) -> bool {
        matches!(tag, Tag::ContextSpecific(_))
    }
}

impl<'a> Encodable for ContextSpecific<'a> {
    fn encoded_len(&self) -> Result<Length> {
        self.value.encoded_len()?.for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        let tag = Tag::ContextSpecific(self.tag_number);
        Header::new(tag, self.value.encoded_len()?)?.encode(encoder)?;
        self.value.encode(encoder)
    }
}

impl<'a> From<&ContextSpecific<'a>> for ContextSpecific<'a> {
    fn from(value: &ContextSpecific<'a>) -> ContextSpecific<'a> {
        *value
    }
}

impl<'a> TryFrom<Any<'a>> for ContextSpecific<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<ContextSpecific<'a>> {
        match any.tag() {
            Tag::ContextSpecific(tag_number) => Ok(Self {
                tag_number,
                value: Any::from_der(any.as_bytes())?,
            }),
            actual => Err(ErrorKind::UnexpectedTag {
                expected: None,
                actual,
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContextSpecific;
    use crate::{Decodable, Encodable, Tag};
    use hex_literal::hex;

    // Public key data from `pkcs8` crate's `ed25519-pkcs8-v2.der`
    const EXAMPLE_BYTES: &[u8] =
        &hex!("A123032100A3A7EAE3A8373830BC47E1167BC50E1DB551999651E0E2DC587623438EAC3F31");

    #[test]
    fn round_trip() {
        let field = ContextSpecific::from_der(EXAMPLE_BYTES).unwrap();
        assert_eq!(field.tag_number.value(), 1);
        assert_eq!(field.value.tag(), Tag::BitString);
        assert_eq!(field.value.as_bytes(), &EXAMPLE_BYTES[5..]);

        let mut buf = [0u8; 128];
        let encoded = field.encode_to_slice(&mut buf).unwrap();
        assert_eq!(encoded, EXAMPLE_BYTES);
    }
}
