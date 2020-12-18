//! ASN.1 `OPTIONAL` as mapped to Rust's `Option` type

use crate::{Any, Decodable, Decoder, Result};
use core::convert::TryFrom;

impl<'a, T> Decodable<'a> for Option<T>
where
    T: TryFrom<Any<'a>>,
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<Option<T>> {
        if decoder.is_empty() {
            Ok(None)
        } else {
            Any::decode(decoder).map(|any| T::try_from(any).ok())
        }
    }
}
