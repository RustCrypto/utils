//! Error types

use core::fmt;

/// Insufficient output buffer length.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InvalidLengthError;

impl fmt::Display for InvalidLengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("insufficient output buffer length")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidLengthError {}

/// Generic error, union of [`InvalidLengthError`] and [`InvalidEncodingError`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// Insufficient output buffer length.
    InvalidEncoding,

    /// Invalid encoding of provided "B64" string.
    InvalidLength,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let s = match self {
            Self::InvalidEncoding => "invalid B64 encoding",
            Self::InvalidLength => "insufficient output buffer length",
        };
        f.write_str(s)
    }
}

impl From<InvalidLengthError> for DecodeError {
    #[inline]
    fn from(_: InvalidLengthError) -> DecodeError {
        DecodeError::InvalidLength
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DecodeError {}
