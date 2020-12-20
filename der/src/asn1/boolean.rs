//! ASN.1 `BOOLEAN` support.

use crate::{Any, Encodable, Encoder, Error, ErrorKind, Header, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// Byte used to encode `true` in ASN.1 DER. From X.690 Section 11.1:
///
/// > If the encoding represents the boolean value TRUE, its single contents
/// > octet shall have all eight bits set to one.
const TRUE_OCTET: u8 = 0b11111111;

/// Byte used to encode `false` in ASN.1 DER.
const FALSE_OCTET: u8 = 0b00000000;

/// ASN.1 `BOOLEAN` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Boolean(bool);

impl Boolean {
    /// Get the ASN.1 DER [`Header`] for this [`Boolean`] value
    pub(crate) fn header(self) -> Header {
        Header {
            tag: Tag::Boolean,
            length: 1u8.into(),
        }
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Boolean {
        Boolean(value)
    }
}

impl From<Boolean> for bool {
    fn from(value: Boolean) -> bool {
        value.0
    }
}

impl TryFrom<Any<'_>> for Boolean {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Boolean> {
        any.tag().assert_eq(Tag::Boolean)?;

        match any.as_bytes() {
            [FALSE_OCTET] => Ok(false.into()),
            [TRUE_OCTET] => Ok(true.into()),
            _ => Err(ErrorKind::Noncanonical.into()),
        }
    }
}

impl Encodable for Boolean {
    fn encoded_len(&self) -> Result<Length> {
        self.header().encoded_len()? + 1u8
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.header().encode(encoder)?;
        let byte = if self.0 { TRUE_OCTET } else { FALSE_OCTET };
        encoder.byte(byte)
    }
}

impl Tagged for Boolean {
    const TAG: Tag = Tag::Boolean;
}
