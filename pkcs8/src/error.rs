//! Error types

use core::fmt;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PKCS#8 error")
    }
}

impl From<const_oid::Error> for Error {
    fn from(_: const_oid::Error) -> Error {
        Error
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Result type
pub type Result<T> = core::result::Result<T, Error>;
