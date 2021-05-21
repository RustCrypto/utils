use core::convert::TryFrom;

use der::{Encodable, Encoder, Tag, Tagged};

pub(crate) struct _AttributesStub;

impl TryFrom<der::Any<'_>> for _AttributesStub {
    type Error = der::Error;

    fn try_from(any: der::Any<'_>) -> der::Result<_AttributesStub> {
        any.tag().assert_eq(Self::TAG)?;

        Ok(_AttributesStub)
    }
}

impl Encodable for _AttributesStub {
    fn encoded_len(&self) -> der::Result<der::Length> {
        der::Length::from(1u8).for_tlv()
    }

    fn encode(&self, _encoder: &mut Encoder<'_>) -> der::Result<()> {
        Ok(())
    }
}

impl Tagged for _AttributesStub {
    const TAG: Tag = Tag::ContextSpecific0;
}
