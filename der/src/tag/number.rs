//! ASN.1 tag numbers

use super::Tag;
use crate::{Error, ErrorKind, Result};
use core::{convert::TryFrom, fmt};

/// ASN.1 tag numbers (i.e. lower 5 bits of a [`Tag`]).
///
/// From X.690 Section 8.1.2.2:
///
/// > bits 5 to 1 shall encode the number of the tag as a binary integer with
/// > bit 5 as the most significant bit.
///
/// This library supports tag numbers ranging from zero to 30 (inclusive),
/// which can be represented as a single identifier octet.
///
/// Section 8.1.2.4 describes how to support multi-byte tag numbers, which are
/// encoded by using a leading tag number of 31 (`0b11111`). This library
/// deliberately does not support this: tag numbers greater than 30 are
/// disallowed.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct TagNumber(pub(super) u8);

impl TagNumber {
    /// Maximum tag number supported (inclusive).
    pub const MAX: u8 = 30;

    /// Create a new tag number (const-friendly).
    ///
    /// Panics if the tag number is greater than [`TagNumber::MAX`]. For a fallible
    /// conversion, use [`TryFrom`] instead.
    #[allow(clippy::no_effect)]
    pub const fn new(byte: u8) -> Self {
        // TODO(tarcieri): hax! use const panic when available
        ["tag number out of range"][(byte > Self::MAX) as usize];
        Self(byte)
    }

    /// Create an `APPLICATION` tag with this tag number.
    pub fn application(self) -> Tag {
        Tag::Application(self)
    }

    /// Create a `CONTEXT-SPECIFIC` tag with this tag number.
    pub fn context_specific(self) -> Tag {
        Tag::ContextSpecific(self)
    }

    /// Create a `PRIVATE` tag with this tag number.
    pub fn private(self) -> Tag {
        Tag::Private(self)
    }

    /// Get the inner value.
    pub fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for TagNumber {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self> {
        match byte {
            0..=Self::MAX => Ok(Self(byte)),
            _ => Err(ErrorKind::UnknownTag { byte }.into()),
        }
    }
}

impl From<TagNumber> for u8 {
    fn from(tag_number: TagNumber) -> u8 {
        tag_number.0
    }
}

impl fmt::Display for TagNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
