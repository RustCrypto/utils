//! Trait definitions

use crate::{Any, Decoder, Error, Result, Tag};
use core::convert::TryFrom;

/// Decoding trait
pub trait Decodable<'a>: Sized {
    /// Attempt to decode this value using the provided decoder
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self>;
}

impl<'a, T> Decodable<'a> for T
where
    T: TryFrom<Any<'a>, Error = Error>,
{
    fn decode(decoder: &mut Decoder<'a>) -> Result<T> {
        Any::decode(decoder).and_then(Self::try_from)
    }
}

/// Types with an associated ASN.1 tag
pub trait Tagged {
    /// ASN.1 tag
    const TAG: Tag;
}
