//! Sequence iterator.

use crate::{Decodable, Decoder, Result};
use core::marker::PhantomData;

/// ASN.1 `SEQUENCE` iterator for [`Sequence`][`super::Sequence`] containing
/// homogeneously typed values.
pub struct SequenceIter<'a, T>
where
    T: Decodable<'a>,
{
    /// Sequence decoder
    decoder: Decoder<'a>,

    /// Type being decoded
    decodable: PhantomData<T>,
}

impl<'a, T> SequenceIter<'a, T>
where
    T: Decodable<'a>,
{
    /// Create a new sequence iterator for the given type.
    pub(super) fn new(decoder: Decoder<'a>) -> Self {
        Self {
            decoder,
            decodable: PhantomData,
        }
    }
}

impl<'a, T> Iterator for SequenceIter<'a, T>
where
    T: Decodable<'a>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Result<T>> {
        if self.decoder.is_finished() {
            None
        } else {
            Some(T::decode(&mut self.decoder))
        }
    }
}
