//! ASN.1 `INTEGER` support.

use crate::{Any, Encodable, Encoder, Error, Header, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `INTEGER` type.
///
/// # Limits
///
/// Presently constrained to `i8`.
///
/// This is not a deliberate decision: the goal of this library is to
/// eventually support other integer types, but they have not yet been
/// implemented.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Integer(i8);

impl Integer {
    /// Get the ASN.1 DER [`Header`] for this [`Integer`] value
    pub(crate) fn header(self) -> Header {
        Header {
            tag: Tag::Integer,
            length: 1u8.into(), // TODO(tarcieri): larger integers
        }
    }
}

impl From<i8> for Integer {
    fn from(x: i8) -> Integer {
        Integer(x)
    }
}

impl From<Integer> for i8 {
    fn from(x: Integer) -> i8 {
        x.0
    }
}

impl TryFrom<Any<'_>> for Integer {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Integer> {
        let tag = any.tag().assert_eq(Tag::Integer)?;

        if any.as_bytes().len() == 1 {
            Ok(Integer(any.as_bytes()[0] as i8))
        } else {
            Err(Error::Length { tag })
        }
    }
}

impl Encodable for Integer {
    fn encoded_len(&self) -> Result<Length> {
        self.header().encoded_len()? + 1u8
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        self.header().encode(encoder)?;
        encoder.byte(self.0 as u8)
    }
}

impl Tagged for Integer {
    const TAG: Tag = Tag::Integer;
}
