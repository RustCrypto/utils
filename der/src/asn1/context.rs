use crate::{
    Any, Choice, Decodable, Decoder, Encodable, Encoder, Header, Length, Result, Tag, Tagged,
};

macro_rules! context {
    ($id:ident, $tag:path, $doc:expr) => {
        #[doc=$doc]
        pub struct $id<T>(pub T);

        impl<T> Tagged for $id<T> {
            const TAG: Tag = $tag;
        }

        impl<'a, T> Decodable<'a> for $id<T>
        where
            T: Choice<'a> + Tagged,
        {
            fn decode(decoder: &mut Decoder<'a>) -> Result<$id<T>> {
                let any = decoder.decode::<Any<'a>>()?;

                if any.tag() == Self::TAG {
                    T::decode(&mut any.as_bytes().into()).map($id)
                } else {
                    Err(crate::ErrorKind::UnexpectedTag {
                        expected: Some(Self::TAG),
                        actual: any.tag(),
                    }
                    .into())
                }
            }
        }

        impl<T> Encodable for $id<T>
        where
            T: Encodable,
        {
            fn encoded_len(&self) -> Result<Length> {
                let inner_len = self.0.encoded_len()?;
                Header::new(Self::TAG, inner_len)?.encoded_len()? + inner_len
            }

            fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
                Header::new(Self::TAG, self.0.encoded_len()?)?.encode(encoder)?;

                self.0.encode(encoder)
            }
        }
    };
}

context! {ContextualTo0, Tag::ContextSpecific0, "A wrapped field contextual to 0"}
context! {ContextualTo1, Tag::ContextSpecific1, "A wrapped field contextual to 1"}
context! {ContextualTo2, Tag::ContextSpecific2, "A wrapped field contextual to 2"}
context! {ContextualTo3, Tag::ContextSpecific3, "A wrapped field contextual to 3"}
