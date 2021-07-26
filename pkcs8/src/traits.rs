//! Traits for parsing objects from PKCS#8 encoded documents

use crate::{PrivateKeyInfo, Result, SubjectPublicKeyInfo};
use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use crate::{PrivateKeyDocument, PublicKeyDocument};

#[cfg(feature = "encryption")]
use {
    crate::{EncryptedPrivateKeyDocument, EncryptedPrivateKeyInfo},
    rand_core::{CryptoRng, RngCore},
};

#[cfg(feature = "pem")]
use {crate::LineEnding, alloc::string::String};

#[cfg(feature = "pkcs1")]
use crate::{Error, ObjectIdentifier};

#[cfg(feature = "std")]
use std::path::Path;

#[cfg(any(feature = "pem", feature = "std"))]
use zeroize::Zeroizing;

#[cfg(all(feature = "alloc", feature = "pkcs1"))]
use crate::AlgorithmIdentifier;

/// PKCS#1 RSA Algorithm [`ObjectIdentifier`].
///
/// <http://oid-info.com/get/1.2.840.113549.1.1.1>
#[cfg(feature = "pkcs1")]
const PKCS1_OID: ObjectIdentifier = ObjectIdentifier::new("1.2.840.113549.1.1.1");

/// Parse a private key object from a PKCS#8 encoded document.
pub trait FromPrivateKey: Sized {
    /// Parse the [`PrivateKeyInfo`] from a PKCS#8-encoded document.
    fn from_pkcs8_private_key_info(private_key_info: PrivateKeyInfo<'_>) -> Result<Self>;

    /// Deserialize PKCS#8 private key from ASN.1 DER-encoded data
    /// (binary format).
    fn from_pkcs8_der(bytes: &[u8]) -> Result<Self> {
        Self::from_pkcs8_private_key_info(PrivateKeyInfo::try_from(bytes)?)
    }

    /// Deserialize encrypted PKCS#8 private key from ASN.1 DER-encoded data
    /// (binary format) and attempt to decrypt it using the provided password.
    #[cfg(feature = "encryption")]
    #[cfg_attr(docsrs, doc(cfg(feature = "encryption")))]
    fn from_pkcs8_encrypted_der(bytes: &[u8], password: impl AsRef<[u8]>) -> Result<Self> {
        EncryptedPrivateKeyInfo::try_from(bytes)?
            .decrypt(password)
            .and_then(|doc| Self::from_pkcs8_doc(&doc))
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

    /// Deserialize encrypted PKCS#8-encoded private key from PEM and attempt
    /// to decrypt it using the provided password.
    ///
    /// Keys in this format begin with the following delimiter:
    ///
    /// ```text
    /// -----BEGIN ENCRYPTED PRIVATE KEY-----
    /// ```
    #[cfg(all(feature = "encryption", feature = "pem"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "encryption")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs8_encrypted_pem(s: &str, password: impl AsRef<[u8]>) -> Result<Self> {
        EncryptedPrivateKeyDocument::from_pem(s)?
            .decrypt(password)
            .and_then(|doc| Self::from_pkcs8_doc(&doc))
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
        Self::from_spki(SubjectPublicKeyInfo::try_from(bytes)?)
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
    fn read_public_key_der_file(path: impl AsRef<Path>) -> Result<Self> {
        PublicKeyDocument::read_der_file(path).and_then(|doc| Self::from_public_key_doc(&doc))
    }

    /// Load public key object from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_public_key_pem_file(path: impl AsRef<Path>) -> Result<Self> {
        PublicKeyDocument::read_pem_file(path).and_then(|doc| Self::from_public_key_doc(&doc))
    }
}

/// Serialize a private key object to a PKCS#8 encoded document.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait ToPrivateKey {
    /// Serialize a [`PrivateKeyDocument`] containing a PKCS#8-encoded private key.
    fn to_pkcs8_der(&self) -> Result<PrivateKeyDocument>;

    /// Create an [`EncryptedPrivateKeyDocument`] containing the ciphertext of
    /// a PKCS#8 encoded private key encrypted under the given `password`.
    #[cfg(feature = "encryption")]
    #[cfg_attr(docsrs, doc(cfg(feature = "encryption")))]
    fn to_pkcs8_encrypted_der(
        &self,
        rng: impl CryptoRng + RngCore,
        password: impl AsRef<[u8]>,
    ) -> Result<EncryptedPrivateKeyDocument> {
        self.to_pkcs8_der()?.encrypt(rng, password)
    }

    /// Serialize this private key as PEM-encoded PKCS#8.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs8_pem(&self) -> Result<Zeroizing<String>> {
        self.to_pkcs8_pem_with_le(LineEnding::default())
    }

    /// Serialize this private key as PEM-encoded PKCS#8 with the given [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs8_pem_with_le(&self, line_ending: LineEnding) -> Result<Zeroizing<String>> {
        Ok(self.to_pkcs8_der()?.to_pem_with_le(line_ending))
    }

    /// Serialize this private key as an encrypted PEM-encoded PKCS#8 private
    /// key using the `provided` to derive an encryption key.
    #[cfg(all(feature = "encryption", feature = "pem"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "encryption")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs8_encrypted_pem(
        &self,
        rng: impl CryptoRng + RngCore,
        password: impl AsRef<[u8]>,
    ) -> Result<Zeroizing<String>> {
        self.to_pkcs8_encrypted_der(rng, password)
            .map(|key| key.to_pem())
    }

    /// Write ASN.1 DER-encoded PKCS#8 private key to the given path
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs8_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_pkcs8_der()?.write_der_file(path)
    }

    /// Write ASN.1 DER-encoded PKCS#8 private key to the given path
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs8_pem_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_pkcs8_der()?.write_pem_file(path)
    }
}

/// Serialize a public key object to a SPKI-encoded document.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait ToPublicKey {
    /// Serialize a [`PublicKeyDocument`] containing a SPKI-encoded public key.
    fn to_public_key_der(&self) -> Result<PublicKeyDocument>;

    /// Serialize this public key as PEM-encoded SPKI.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_public_key_pem(&self) -> Result<String> {
        self.to_public_key_pem_with_le(LineEnding::default())
    }

    /// Serialize this public key as PEM-encoded SPKI with the given [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_public_key_pem_with_le(&self, line_ending: LineEnding) -> Result<String> {
        Ok(self.to_public_key_der()?.to_pem_with_le(line_ending))
    }

    /// Write ASN.1 DER-encoded public key to the given path
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_public_key_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_public_key_der()?.write_der_file(path)
    }

    /// Write ASN.1 DER-encoded public key to the given path
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_public_key_pem_file(&self, path: impl AsRef<Path>) -> Result<()> {
        self.to_public_key_der()?.write_pem_file(path)
    }
}

#[cfg(feature = "pkcs1")]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs1")))]
impl<K: pkcs1::FromRsaPrivateKey> FromPrivateKey for K {
    fn from_pkcs8_private_key_info(pkcs8_key: PrivateKeyInfo<'_>) -> Result<Self> {
        pkcs8_key.algorithm.assert_algorithm_oid(PKCS1_OID)?;

        if pkcs8_key.algorithm.parameters != Some(der::asn1::Null.into()) {
            return Err(Error::ParametersMalformed);
        }

        let pkcs1_key = pkcs1::RsaPrivateKey::try_from(pkcs8_key.private_key)?;
        Ok(K::from_pkcs1_private_key(pkcs1_key)?)
    }
}

#[cfg(feature = "pkcs1")]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs1")))]
impl<K: pkcs1::FromRsaPublicKey> FromPublicKey for K {
    fn from_spki(pkcs8_key: SubjectPublicKeyInfo<'_>) -> Result<Self> {
        pkcs8_key.algorithm.assert_algorithm_oid(PKCS1_OID)?;

        if pkcs8_key.algorithm.parameters != Some(der::asn1::Null.into()) {
            return Err(Error::ParametersMalformed);
        }

        let pkcs1_key = pkcs1::RsaPublicKey::try_from(pkcs8_key.subject_public_key)?;
        Ok(K::from_pkcs1_public_key(pkcs1_key)?)
    }
}

#[cfg(all(feature = "alloc", feature = "pkcs1"))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs1")))]
impl<K: pkcs1::ToRsaPrivateKey> ToPrivateKey for K {
    fn to_pkcs8_der(&self) -> Result<PrivateKeyDocument> {
        let pkcs1_der = self.to_pkcs1_der()?;

        let algorithm = AlgorithmIdentifier {
            oid: PKCS1_OID,
            parameters: Some(der::asn1::Null.into()),
        };

        Ok(PrivateKeyInfo::new(algorithm, pkcs1_der.as_ref()).to_der())
    }
}

#[cfg(all(feature = "alloc", feature = "pkcs1"))]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs1")))]
impl<K: pkcs1::ToRsaPublicKey> ToPublicKey for K {
    fn to_public_key_der(&self) -> Result<PublicKeyDocument> {
        let pkcs1_der = self.to_pkcs1_der()?;

        let algorithm = AlgorithmIdentifier {
            oid: PKCS1_OID,
            parameters: Some(der::asn1::Null.into()),
        };

        Ok(SubjectPublicKeyInfo {
            algorithm,
            subject_public_key: pkcs1_der.as_ref(),
        }
        .into())
    }
}
