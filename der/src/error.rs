//! Error types.

use core::fmt;

/// Result type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ASN.1 error")
    }
}

#[cfg(feature = "oid")]
impl From<oid::Error> for Error {
    fn from(_: oid::Error) -> Error {
        Error
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
