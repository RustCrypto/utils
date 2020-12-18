//! ASN.1 `SEQUENCE`

use crate::{Any, Decoder, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `SEQUENCE`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Sequence<'a> {
    /// Inner value
    inner: &'a [u8],
}

impl<'a> Sequence<'a> {
    /// Create a new [`Sequence`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        if slice.len() <= Length::max() {
            Ok(Self { inner: slice })
        } else {
            Err(Error::Length { tag: Tag::Sequence })
        }
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner
    }

    /// Obtain a [`Decoder`] for the data in this [`Sequence`]
    pub fn decoder(&self) -> Decoder<'a> {
        self.inner
    }
}

impl AsRef<[u8]> for Sequence<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> TryFrom<Any<'a>> for Sequence<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Sequence<'a>> {
        any.tag().expect(Tag::Sequence)?;
        Self::new(any.value())
    }
}

impl<'a> Tagged for Sequence<'a> {
    const TAG: Tag = Tag::Sequence;
}
