//! Context-specific field.

use crate::{
    Any, Choice, Decodable, Decoder, Encodable, Encoder, ErrorKind, Header, Length, Result, Tag,
};

/// Context-specific field.
///
/// This type encodes a field which is specific to a particular context,
/// and has a special "context-specific tag" (presently 0-15 supported).
///
/// Any context-specific field can be decoded/encoded with this type.
/// The intended use is to dynamically dispatch off of the context-specific
/// tag when decoding, which allows support for extensions, which are denoted
/// in an ASN.1 schema using the `...` ellipsis extension marker.
///
///
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ContextSpecific<'a> {
    /// Context-specific tag value sans the leading `0b10000000` class
    /// identifier bit and `0b100000` constructed flag.
    pub(crate) tag: u8,

    /// Value of the field.
    pub(crate) value: Any<'a>,
}

impl<'a> ContextSpecific<'a> {
    /// Create a new context-specific field.
    ///
    /// The tag value includes only lower 6-bits of the context specific tag,
    /// sans the leading `10` high bits identifying the context-specific tag
    /// class as well as the constructed flag.
    pub fn new(tag: u8, value: Any<'a>) -> Result<Self> {
        // Ensure we consider the context-specific tag valid
        Tag::context_specific(tag)?;

        Ok(Self { tag, value })
    }

    /// Get the context-specific tag for this field.
    ///
    /// The tag value includes only lower 6-bits of the context specific tag,
    /// sans the leading `10` high bits identifying the context-specific tag
    /// class as well as the constructed flag.
    pub fn tag(self) -> u8 {
        self.tag
    }

    /// Get the value of this context-specific tag.
    pub fn value(self) -> Any<'a> {
        self.value
    }
}

impl<'a> Choice<'a> for ContextSpecific<'a> {
    fn can_decode(tag: Tag) -> bool {
        tag.is_context_specific()
    }
}

impl<'a> Decodable<'a> for ContextSpecific<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self> {
        let header = Header::decode(decoder)?;

        let tag = if header.tag.is_context_specific() {
            (header.tag as u8)
                .checked_sub(0xA0)
                .ok_or(ErrorKind::Overflow)?
        } else {
            return decoder.error(ErrorKind::UnexpectedTag {
                expected: None,
                actual: header.tag,
            });
        };

        Self::new(tag, decoder.any()?)
    }
}

impl<'a> Encodable for ContextSpecific<'a> {
    fn encoded_len(&self) -> Result<Length> {
        self.value.encoded_len()?.for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        let tag = Tag::context_specific(self.tag)?;
        Header::new(tag, self.value.encoded_len()?)?.encode(encoder)?;
        self.value.encode(encoder)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn round_trip() {}
}
