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

    /// General I/O errors
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    Io,

    /// File not found error
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    FileNotFound,

    /// Permission denied reading file
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    PermissionDenied,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Decode => f.write_str("PKCS#8 decoding error"),
            Error::Encode => f.write_str("PKCS#8 encoding error"),
            #[cfg(feature = "std")]
            Error::Io => f.write_str("I/O error"),
            #[cfg(feature = "std")]
            Error::FileNotFound => f.write_str("file not found"),
            #[cfg(feature = "std")]
            Error::PermissionDenied => f.write_str("permission denied"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<const_oid::Error> for Error {
    fn from(_: const_oid::Error) -> Error {
        Error::Decode
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        match err.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound,
            std::io::ErrorKind::PermissionDenied => Error::PermissionDenied,
            _ => Error::Io,
        }
    }
}
