//! Error types

use core::fmt;

#[cfg(feature = "pem")]
use crate::pem;

/// Message to display when an `expect`-ed DER encoding error occurs
#[cfg(feature = "alloc")]
pub(crate) const DER_ENCODING_MSG: &str = "DER encoding error";

/// Result type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// ASN.1 DER-related errors.
    Asn1(der::Error),

    /// Cryptographic errors.
    ///
    /// These can be used by RSA implementations to signal that a key is
    /// invalid for cryptographic reasons. This means the document parsed
    /// correctly, but one of the values contained within was invalid, e.g.
    /// a number expected to be a prime was not a prime.
    Crypto,

    /// File not found error.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    FileNotFound,

    /// I/O errors.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    Io,

    /// PEM encoding errors.
    #[cfg(feature = "pem")]
    Pem(pem::Error),

    /// Permission denied reading file.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    PermissionDenied,

    /// Version errors
    Version,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Asn1(err) => write!(f, "PKCS#1 ASN.1 error: {}", err),
            Error::Crypto => f.write_str("PKCS#1 cryptographic error"),
            #[cfg(feature = "std")]
            Error::FileNotFound => f.write_str("file not found"),
            #[cfg(feature = "std")]
            Error::Io => f.write_str("I/O error"),
            #[cfg(feature = "pem")]
            Error::Pem(err) => write!(f, "PKCS#1 {}", err),
            Error::Version => f.write_str("PKCS#1 version error"),
            #[cfg(feature = "std")]
            Error::PermissionDenied => f.write_str("permission denied"),
        }
    }
}

#[cfg(feature = "pem")]
impl From<pem_rfc7468::Error> for Error {
    fn from(err: pem_rfc7468::Error) -> Error {
        Error::Pem(err)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<der::Error> for Error {
    fn from(err: der::Error) -> Error {
        Error::Asn1(err)
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
