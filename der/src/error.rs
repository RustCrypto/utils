//! Error types.

use crate::Tag;
use core::fmt;

/// Result type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// Incorrect length
    Length {
        /// Tag type of the value being decoded
        tag: Tag,
    },

    /// Message is not canonically encoded
    Noncanonical,

    /// Malformed OID
    Oid,

    /// Integer overflow occurred (library bug!)
    Overflow,

    /// Message is longer than this library's internal limits support
    Overlength,

    /// Message is truncated
    Truncated,

    /// Unexpected tag
    UnexpectedTag {
        /// Tag the decoder was expecting (if there is a single such tag).
        ///
        /// `None` if multiple tags are expected/allowed, but the `actual` tag
        /// does not match any of them.
        expected: Option<Tag>,

        /// Actual tag encountered in the message
        actual: Tag,
    },

    /// Unknown/unsupported tag
    UnknownTag {
        /// Raw byte value of the tag
        byte: u8,
    },

    /// Unexpected value
    Value {
        /// Tag of the unexpected value
        tag: Tag,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(tarcieri): real `Display` impl with good error messages
        f.write_str("ASN.1 error")
    }
}

#[cfg(feature = "oid")]
impl From<const_oid::Error> for Error {
    fn from(_: const_oid::Error) -> Error {
        Error::Oid
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
