//! ASN.1 `BIT STRING`

use crate::{Any, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `BIT STRING`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct BitString<'a> {
    /// Inner value
    inner: &'a [u8],
}

impl<'a> BitString<'a> {
    /// Create a new [`BitString`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        if slice.len() <= Length::max() {
            Ok(Self { inner: slice })
        } else {
            Err(Error::Length {
                tag: Tag::BitString,
            })
        }
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner
    }
}

impl AsRef<[u8]> for BitString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> TryFrom<Any<'a>> for BitString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BitString<'a>> {
        any.tag().expect(Tag::BitString)?;
        Self::new(any.value())
    }
}

impl<'a> Tagged for BitString<'a> {
    const TAG: Tag = Tag::BitString;
}
