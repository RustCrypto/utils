use core::convert::TryFrom;

use der::{Any, Encodable, Encoder, Error, Length, Result, Tag, Tagged};

// TODO: make proper attributes struct
#[derive(Debug)]
pub(crate) struct Attributes;

impl TryFrom<Any<'_>> for Attributes {
    type Error = Error;

    fn try_from(any: Any<'_>) -> Result<Attributes> {
        any.tag().assert_eq(Self::TAG)?;

        Ok(Attributes)
    }
}

impl Encodable for Attributes {
    fn encoded_len(&self) -> Result<Length> {
        Length::ONE.for_tlv()
    }

    fn encode(&self, _encoder: &mut Encoder<'_>) -> Result<()> {
        Ok(())
    }
}

impl Tagged for Attributes {
    const TAG: Tag = Tag::ContextSpecific0;
}
