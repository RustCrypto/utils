//! ASN.1 `INTEGER`

use crate::{Any, Error, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `INTEGER`
///
/// # Limits
///
/// Presently constrained to 1-byte values!
///
/// TODO: larger integers
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Integer(u8);

impl From<u8> for Integer {
    fn from(i: u8) -> Integer {
        Integer(i)
    }
}

impl From<Integer> for u8 {
    fn from(i: Integer) -> u8 {
        i.0
    }
}

impl TryFrom<Any<'_>> for Integer {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Integer> {
        let tag = any.tag().expect(Tag::Integer)?;

        if any.value().len() == 1 {
            Ok(any.value()[0].into())
        } else {
            Err(Error::Length { tag })
        }
    }
}

impl Tagged for Integer {
    const TAG: Tag = Tag::Integer;
}
