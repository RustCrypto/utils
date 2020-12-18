//! ASN.1 `OBJECT IDENTIFIER`

pub use const_oid::ObjectIdentifier;

use crate::{Any, Error, Result, Tag, Tagged};
use core::convert::TryFrom;

impl TryFrom<Any<'_>> for ObjectIdentifier {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<ObjectIdentifier> {
        any.tag().expect(Tag::ObjectIdentifier)?;
        Ok(ObjectIdentifier::from_ber(any.value())?)
    }
}

impl<'a> Tagged for ObjectIdentifier {
    const TAG: Tag = Tag::ObjectIdentifier;
}
