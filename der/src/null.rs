//! ASN.1 `NULL`

use crate::{Any, Error, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `NULL`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Null;

impl TryFrom<Any<'_>> for Null {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Null> {
        let tag = any.tag().expect(Tag::Null)?;

        if any.value().is_empty() {
            Ok(Null)
        } else {
            Err(Error::Length { tag })
        }
    }
}

impl Tagged for Null {
    const TAG: Tag = Tag::Integer;
}
