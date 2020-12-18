//! ASN.1 `OBJECT IDENTIFIER`

use crate::{
    Any, Encodable, Encoder, Error, Header, Length, ObjectIdentifier, Result, Tag, Tagged,
};
use core::convert::TryFrom;

impl TryFrom<Any<'_>> for ObjectIdentifier {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<ObjectIdentifier> {
        any.tag().expect(Tag::ObjectIdentifier)?;
        Ok(ObjectIdentifier::from_ber(any.as_bytes())?)
    }
}

impl Encodable for ObjectIdentifier {
    fn encoded_len(&self) -> Result<Length> {
        let ber_len = self.ber_len();
        let header = Header::new(Tag::ObjectIdentifier, ber_len)?;
        header.encoded_len()? + ber_len
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        encoder.oid(*self)
    }
}

impl<'a> Tagged for ObjectIdentifier {
    const TAG: Tag = Tag::ObjectIdentifier;
}
