//! Common handling for types backed by `str` slices with enforcement of a
//! library-level length limitation i.e. `Length::max()`.

use crate::{Length, Result};
use core::{convert::TryFrom, str};

/// String slice newtype which respects the [`Length::max`] limit.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) struct StrSlice<'a> {
    /// Inner value
    pub(crate) inner: &'a str,

    /// Precomputed `Length` (avoids possible panicking conversions)
    pub(crate) length: Length,
}

impl<'a> StrSlice<'a> {
    /// Create a new [`StrSlice`], ensuring that the byte representation of
    /// the provided `str` value is shorter than `Length::max()`.
    pub fn new(s: &'a str) -> Result<Self> {
        Ok(Self {
            inner: s,
            length: Length::try_from(s.as_bytes().len())?,
        })
    }

    /// Parse a [`StrSlice`] from UTF-8 encoded bytes.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        Self::new(str::from_utf8(bytes)?)
    }

    /// Borrow the inner `str`
    pub fn as_str(&self) -> &'a str {
        self.inner
    }

    /// Borrow the inner byte slice
    pub fn as_bytes(&self) -> &'a [u8] {
        self.inner.as_bytes()
    }

    /// Get the [`Length`] of this [`StrSlice`]
    pub fn len(self) -> Length {
        self.length
    }

    /// Is this [`StrSlice`] empty?
    pub fn is_empty(self) -> bool {
        self.len() == Length::ZERO
    }
}

impl AsRef<str> for StrSlice<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for StrSlice<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}
