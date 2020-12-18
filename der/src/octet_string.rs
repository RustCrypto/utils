//! ASN.1 `OCTET STRING`

use crate::{Any, Error, Length, Result, Tag, Tagged};
use core::convert::TryFrom;

/// ASN.1 `OCTET STRING`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OctetString<'a> {
    /// Inner value
    inner: &'a [u8],
}

impl<'a> OctetString<'a> {
    /// Create a new [`OctetString`] from a slice
    pub fn new(slice: &'a [u8]) -> Result<Self> {
        if slice.len() <= Length::max() {
            Ok(Self { inner: slice })
        } else {
            Err(Error::Length {
                tag: Tag::OctetString,
            })
        }
    }

    /// Borrow the inner byte sequence
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner
    }
}

impl AsRef<[u8]> for OctetString<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> TryFrom<Any<'a>> for OctetString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<OctetString<'a>> {
        any.tag().expect(Tag::OctetString)?;
        Self::new(any.value())
    }
}

impl<'a> Tagged for OctetString<'a> {
    const TAG: Tag = Tag::OctetString;
}
