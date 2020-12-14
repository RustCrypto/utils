//! Traits for parsing objects from PKCS#8 encoded documents

use crate::{PrivateKeyInfo, Result, SubjectPublicKeyInfo};

#[cfg(feature = "alloc")]
use crate::{PrivateKeyDocument, PublicKeyDocument};

#[cfg(feature = "pem")]
use alloc::string::String;

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(any(feature = "pem", feature = "std"))]
use zeroize::Zeroizing;

/// Parse a private key object from a PKCS#8 encoded document.
pub trait FromPrivateKey: Sized {
    /// Parse the [`PrivateKeyInfo`] from a PKCS#8-encoded document.
    fn from_pkcs8_private_key_info(private_key_info: PrivateKeyInfo<'_>) -> Result<Self>;

    /// Deserialize PKCS#8 private key from ASN.1 DER-encoded data
    /// (binary format).
    fn from_pkcs8_der(bytes: &[u8]) -> Result<Self> {
        PrivateKeyInfo::from_der(bytes).and_then(Self::from_pkcs8_private_key_info)
    }

    /// Deserialize PKCS#8 private key from a [`PrivateKeyDocument`].
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn from_pkcs8_doc(doc: &PrivateKeyDocument) -> Result<Self> {
        Self::from_pkcs8_private_key_info(doc.private_key_info())
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
        PrivateKeyDocument::from_pem(s).and_then(|doc| Self::from_pkcs8_doc(&doc))
    }

    /// Load PKCS#8 private key from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs8_der_file(path: impl AsRef<Path>) -> Result<Self> {
        PrivateKeyDocument::read_der_file(path).and_then(|doc| Self::from_pkcs8_doc(&doc))
    }

    /// Load PKCS#8 private key from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs8_pem_file(path: impl AsRef<Path>) -> Result<Self> {
        PrivateKeyDocument::read_pem_file(path).and_then(|doc| Self::from_pkcs8_doc(&doc))
    }
}

/// Parse a public key object from an encoded SPKI document.
pub trait FromPublicKey: Sized {
    /// Parse [`SubjectPublicKeyInfo`] into a public key object.
    fn from_spki(spki: SubjectPublicKeyInfo<'_>) -> Result<Self>;

    /// Deserialize object from ASN.1 DER-encoded [`SubjectPublicKeyInfo`]
    /// (binary format).
    fn from_public_key_der(bytes: &[u8]) -> Result<Self> {
        SubjectPublicKeyInfo::from_der(bytes).and_then(Self::from_spki)
    }

    /// Deserialize PKCS#8 private key from a [`PrivateKeyDocument`].
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn from_public_key_doc(doc: &PublicKeyDocument) -> Result<Self> {
        Self::from_spki(doc.spki())
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
        PublicKeyDocument::from_pem(s).and_then(|doc| Self::from_public_key_doc(&doc))
    }

    /// Load public key object from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_public_key_der_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        PublicKeyDocument::read_der_file(path).and_then(|doc| Self::from_public_key_doc(&doc))
    }

    /// Load public key object from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_public_key_pem_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        PublicKeyDocument::read_pem_file(path).and_then(|doc| Self::from_public_key_doc(&doc))
    }
}

/// Serialize a private key object to a PKCS#8 encoded document.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait ToPrivateKey {
    /// Serialize a [`PrivateKeyDocument`] containing a PKCS#8-encoded private key.
    fn to_pkcs8_der(&self) -> PrivateKeyDocument;

    /// Serialize this private key as PEM-encoded PKCS#8.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs8_pem(&self) -> Zeroizing<String> {
        self.to_pkcs8_der().to_pem()
    }

    /// Write ASN.1 DER-encoded PKCS#8 private key to the given path
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs8_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_pkcs8_der().write_der_file(path)
    }

    /// Write ASN.1 DER-encoded PKCS#8 private key to the given path
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs8_pem_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_pkcs8_der().write_pem_file(path)
    }
}

/// Serialize a public key object to a SPKI-encoded document.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait ToPublicKey {
    /// Serialize a [`PublicKeyDocument`] containing a SPKI-encoded public key.
    fn to_public_key_der(&self) -> PublicKeyDocument;

    /// Serialize this public key as PEM-encoded SPKI.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_public_key_pem(&self) -> String {
        self.to_public_key_der().to_pem()
    }

    /// Write ASN.1 DER-encoded public key to the given path
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_public_key_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_public_key_der().write_der_file(path)
    }

    /// Write ASN.1 DER-encoded public key to the given path
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_public_key_pem_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_public_key_der().write_pem_file(path)
    }
}
