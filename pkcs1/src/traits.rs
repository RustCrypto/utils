//! Traits for parsing objects from PKCS#1 encoded documents

use crate::{Result, RsaPrivateKey, RsaPublicKey};
use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use crate::{RsaPrivateKeyDocument, RsaPublicKeyDocument};

#[cfg(feature = "pem")]
use {crate::LineEnding, alloc::string::String};

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(any(feature = "pem", feature = "std"))]
use zeroize::Zeroizing;

/// Parse an [`RsaPrivateKey`] from a PKCS#1-encoded document.
pub trait FromRsaPrivateKey: Sized {
    /// Parse the [`RsaPrivateKey`] from a PKCS#1-encoded document.
    fn from_pkcs1_private_key(private_key: RsaPrivateKey<'_>) -> Result<Self>;

    /// Deserialize PKCS#1 private key from ASN.1 DER-encoded data
    /// (binary format).
    fn from_pkcs1_der(bytes: &[u8]) -> Result<Self> {
        Self::from_pkcs1_private_key(RsaPrivateKey::try_from(bytes)?)
    }

    /// Deserialize PKCS#1-encoded private key from PEM.
    ///
    /// Keys in this format begin with the following:
    ///
    /// ```text
    /// -----BEGIN RSA PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs1_pem(s: &str) -> Result<Self> {
        RsaPrivateKeyDocument::from_pkcs1_pem(s)
            .and_then(|doc| Self::from_pkcs1_private_key(doc.private_key()))
    }

    /// Load PKCS#1 private key from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs1_der_file(path: &Path) -> Result<Self> {
        RsaPrivateKeyDocument::read_pkcs1_der_file(path)
            .and_then(|doc| Self::from_pkcs1_private_key(doc.private_key()))
    }

    /// Load PKCS#1 private key from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs1_pem_file(path: &Path) -> Result<Self> {
        RsaPrivateKeyDocument::read_pkcs1_pem_file(path)
            .and_then(|doc| Self::from_pkcs1_private_key(doc.private_key()))
    }
}

/// Parse a [`RsaPublicKey`] from a PKCS#1-encoded document.
pub trait FromRsaPublicKey: Sized {
    /// Parse [`RsaPublicKey`] into a [`RsaPublicKey`].
    fn from_pkcs1_public_key(public_key: RsaPublicKey<'_>) -> Result<Self>;

    /// Deserialize object from ASN.1 DER-encoded [`RsaPublicKey`]
    /// (binary format).
    fn from_pkcs1_der(bytes: &[u8]) -> Result<Self> {
        Self::from_pkcs1_public_key(RsaPublicKey::try_from(bytes)?)
    }

    /// Deserialize PEM-encoded [`RsaPublicKey`].
    ///
    /// Keys in this format begin with the following:
    ///
    /// ```text
    /// -----BEGIN RSA PUBLIC KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs1_pem(s: &str) -> Result<Self> {
        RsaPublicKeyDocument::from_pkcs1_pem(s)
            .and_then(|doc| Self::from_pkcs1_public_key(doc.public_key()))
    }

    /// Load [`RsaPublicKey`] from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs1_der_file(path: &Path) -> Result<Self> {
        RsaPublicKeyDocument::read_pkcs1_der_file(path)
            .and_then(|doc| Self::from_pkcs1_public_key(doc.public_key()))
    }

    /// Load [`RsaPublicKey`] from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs1_pem_file(path: &Path) -> Result<Self> {
        RsaPublicKeyDocument::read_pkcs1_pem_file(path)
            .and_then(|doc| Self::from_pkcs1_public_key(doc.public_key()))
    }
}

/// Serialize a [`RsaPrivateKey`] to a PKCS#1 encoded document.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait ToRsaPrivateKey {
    /// Serialize a [`RsaPrivateKeyDocument`] containing a PKCS#1-encoded private key.
    fn to_pkcs1_der(&self) -> Result<RsaPrivateKeyDocument>;

    /// Serialize this private key as PEM-encoded PKCS#1.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs1_pem(&self) -> Result<Zeroizing<String>> {
        self.to_pkcs1_pem_with_le(LineEnding::default())
    }

    /// Serialize this private key as PEM-encoded PKCS#1 with the given [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs1_pem_with_le(&self, line_ending: LineEnding) -> Result<Zeroizing<String>> {
        self.to_pkcs1_der()?.to_pkcs1_pem_with_le(line_ending)
    }

    /// Write ASN.1 DER-encoded PKCS#1 private key to the given path.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_der_file(&self, path: &Path) -> Result<()> {
        self.to_pkcs1_der()?.write_pkcs1_der_file(path)
    }

    /// Write ASN.1 DER-encoded PKCS#1 private key to the given path.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_pem_file(&self, path: &Path) -> Result<()> {
        self.to_pkcs1_der()?.write_pkcs1_pem_file(path)
    }
}

/// Serialize a [`RsaPublicKey`] to a PKCS#1-encoded document.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait ToRsaPublicKey {
    /// Serialize a [`RsaPublicKeyDocument`] containing a PKCS#1-encoded public key.
    fn to_pkcs1_der(&self) -> Result<RsaPublicKeyDocument>;

    /// Serialize this public key as PEM-encoded PKCS#1.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs1_pem(&self) -> Result<String> {
        self.to_pkcs1_pem_with_le(LineEnding::default())
    }

    /// Serialize this public key as PEM-encoded PKCS#1 with the given line ending.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs1_pem_with_le(&self, line_ending: LineEnding) -> Result<String> {
        self.to_pkcs1_der()?.to_pkcs1_pem_with_le(line_ending)
    }

    /// Write ASN.1 DER-encoded public key to the given path.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_der_file(&self, path: &Path) -> Result<()> {
        self.to_pkcs1_der()?.write_pkcs1_der_file(path)
    }

    /// Write ASN.1 DER-encoded public key to the given path.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_pem_file(&self, path: &Path) -> Result<()> {
        self.to_pkcs1_der()?.write_pkcs1_pem_file(path)
    }
}
