//! Traits for parsing objects from PKCS#8 encoded documents

use crate::{PrivateKeyInfo, Result, SubjectPublicKeyInfo};

#[cfg(feature = "pem")]
use crate::{PrivateKeyDocument, PublicKeyDocument};

#[cfg(feature = "std")]
use {crate::Error, std::vec::Vec, zeroize::Zeroizing};

/// Parse a private key object from a PKCS#8 encoded document.
pub trait FromPrivateKey: Sized {
    /// Parse the `PrivateKeyInfo` from a PKCS#8-encoded document.
    fn from_pkcs8_private_key_info(private_key_info: PrivateKeyInfo<'_>) -> Result<Self>;

    /// Deserialize PKCS#8 private key from ASN.1 DER-encoded data
    /// (binary format).
    fn from_pkcs8_der(bytes: &[u8]) -> Result<Self> {
        Self::from_pkcs8_private_key_info(PrivateKeyInfo::from_der(bytes)?)
    }

    /// Deserialize PKCS#8-encoded private key from PEM.
    ///
    /// Keys in this format begin with the following delimiter:
    ///
    /// ```text
    /// -----BEGIN PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs8_pem(s: &str) -> Result<Self> {
        Self::from_pkcs8_der(PrivateKeyDocument::from_pem(s)?.as_ref())
    }

    /// Load PKCS#8 private key from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn load_pkcs8_der(path: impl AsRef<std::path::Path>) -> Result<Self> {
        load_file(path).and_then(|bytes| Self::from_pkcs8_der(&*bytes))
    }

    /// Load PKCS#8 private key from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn load_pkcs8_pem(path: impl AsRef<std::path::Path>) -> Result<Self> {
        load_file(path).and_then(|bytes| {
            let pem = std::str::from_utf8(&*bytes).map_err(|_| Error)?;
            Self::from_pkcs8_pem(pem)
        })
    }
}

/// Parse a public key object from an encoded SPKI document.
pub trait FromPublicKey: Sized {
    /// Parse [`SubjectPublicKeyInfo`] into a public key object.
    fn from_spki(spki: SubjectPublicKeyInfo<'_>) -> Result<Self>;

    /// Deserialize object from ASN.1 DER-encoded [`SubjectPublicKeyInfo`]
    /// (binary format).
    fn from_public_key_der(bytes: &[u8]) -> Result<Self> {
        Self::from_spki(SubjectPublicKeyInfo::from_der(bytes)?)
    }

    /// Deserialize PEM-encoded [`SubjectPublicKeyInfo`].
    ///
    /// Keys in this format begin with the following delimiter:
    ///
    /// ```text
    /// -----BEGIN PUBLIC KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_public_key_pem(s: &str) -> Result<Self> {
        Self::from_public_key_der(PublicKeyDocument::from_pem(s)?.as_ref())
    }

    /// Load public key object from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn load_public_key_der(path: impl AsRef<std::path::Path>) -> Result<Self> {
        load_file(path).and_then(|bytes| Self::from_public_key_der(&*bytes))
    }

    /// Load public key object from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn load_public_key_pem(path: impl AsRef<std::path::Path>) -> Result<Self> {
        load_file(path).and_then(|bytes| {
            let pem = std::str::from_utf8(&*bytes).map_err(|_| Error)?;
            Self::from_public_key_pem(pem)
        })
    }
}

/// Load data from a file into a self-zeroizing buffer
#[cfg(feature = "std")]
fn load_file(path: impl AsRef<std::path::Path>) -> Result<Zeroizing<Vec<u8>>> {
    std::fs::read(path).map(Zeroizing::new).map_err(|_| Error)
}
