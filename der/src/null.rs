//! ASN.1 `NULL` support.

use crate::{Any, Encodable, Encoder, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `NULL` type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Null;

impl TryFrom<Any<'_>> for Null {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Null> {
        let tag = any.tag().expect(Tag::Null)?;

        if any.is_empty() {
            Ok(Null)
        } else {
            Err(Error::Length { tag })
        }
    }
}

impl<'a> From<Null> for Any<'a> {
    fn from(_: Null) -> Any<'a> {
        Any::new(Tag::Null, &[]).unwrap()
    }
}

impl Encodable for Null {
    fn encoded_len(&self) -> Result<Length> {
        Any::from(*self).encoded_len()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Any::from(*self).encode(encoder)
    }
}

impl Tagged for Null {
    const TAG: Tag = Tag::Integer;
}
