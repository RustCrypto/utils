//! ASN.1 `OPTIONAL` as mapped to Rust's `Option` type

use crate::{Decodable, Decoder, Encodable, Encoder, Length, Result};

impl<'a, T> Decodable<'a> for Option<T>
where
    T: Decodable<'a>,
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<Option<T>> {
        if decoder.is_finished() {
            Ok(None)
        } else {
            T::decode(decoder).map(Some)
        }
    }
}

impl<T> Encodable for Option<T>
where
    T: Encodable,
{
    fn encoded_len(&self) -> Result<Length> {
        if let Some(encodable) = self {
            encodable.encoded_len()
        } else {
            Ok(0u8.into())
        }
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        if let Some(encodable) = self {
            encodable.encode(encoder)
        } else {
            Ok(())
        }
    }
}
