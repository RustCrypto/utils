//! Error types

use core::fmt;

/// Message to display when an `expect`-ed DER encoding error occurs
#[cfg(feature = "alloc")]
pub(crate) const DER_ENCODING_MSG: &str = "DER encoding error";

/// Result type
pub type Result<T> = core::result::Result<T, Error>;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// ASN.1 DER-related errors
    Asn1(der::Error),

    /// Cryptographic errors
    Crypto,

    /// PEM encoding errors
    #[cfg(feature = "pem")]
    Pem,

    /// I/O errors
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
            Error::Crypto => f.write_str("PKCS#8 cryptographic error"),
            Error::Asn1(err) => write!(f, "PKCS#8 ASN.1 error: {}", err),
            #[cfg(feature = "pem")]
            Error::Pem => f.write_str("PKCS#8 PEM error"),
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

impl From<der::Error> for Error {
    fn from(err: der::Error) -> Error {
        Error::Asn1(err)
    }
}

impl From<der::ErrorKind> for Error {
    fn from(err: der::ErrorKind) -> Error {
        Error::Asn1(err.into())
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
