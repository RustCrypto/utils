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

/// Invalid encoding of provided "B64" string.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct InvalidEncodingError;

impl fmt::Display for InvalidEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("insufficient output buffer length")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidEncodingError {}

/// Generic error, union of [`InvalidLengthError`] and [`InvalidEncodingError`].
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Insufficient output buffer length.
    InvalidEncoding,
    /// Invalid encoding of provided "B64" string.
    InvalidLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            v @ Self::InvalidEncoding => v.fmt(f),
            v @ Self::InvalidLength => v.fmt(f),
        }
    }
}

impl From<InvalidEncodingError> for Error {
    #[inline]
    fn from(_: InvalidEncodingError) -> Error {
        Error::InvalidEncoding
    }
}

impl From<InvalidLengthError> for Error {
    #[inline]
    fn from(_: InvalidLengthError) -> Error {
        Error::InvalidLength
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

