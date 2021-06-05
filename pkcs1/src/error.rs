//! Error types

use core::fmt;

/// Result type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// Decoding errors
    Decode,

    /// Encoding errors
    Encode,

    /// Version errors
    Version,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::Decode => "PKCS#1 decoding error",
            Error::Encode => "PKCS#1 encoding error",
            Error::Version => "PKCS#1 version error",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<der::Error> for Error {
    fn from(_: der::Error) -> Error {
        Error::Decode
    }
}
