//! ASN.1 `OPTIONAL` as mapped to Rust's `Option` type

use crate::{Decodable, Decoder, Result};

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
