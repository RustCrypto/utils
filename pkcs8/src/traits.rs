//! Traits for parsing objects from PKCS#8 encoded documents

use crate::{PrivateKeyInfo, Result, SubjectPublicKeyInfo};

#[cfg(feature = "pem")]
use crate::{PrivateKeyDocument, PublicKeyDocument};

/// Parse a private key object from a PKCS#8 encoded document.
pub trait FromPrivateKey: Sized {
    /// Parse the `PrivateKeyInfo` from a PKCS#8-encoded document.
    fn from_pkcs8_private_key_info(private_key_info: PrivateKeyInfo<'_>) -> Result<Self>;

    /// Deserialize PKCS#8-encoded private key from ASN.1 DER
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
}
