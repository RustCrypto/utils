//! PKCS#8 documents: serialized PKCS#8 private keys and SPKI public keys
// TODO(tarcieri): heapless support?

use crate::{Error, PrivateKeyInfo, Result, SubjectPublicKeyInfo};
use alloc::{borrow::ToOwned, vec::Vec};
use core::{convert::TryFrom, fmt};
use zeroize::Zeroizing;

#[cfg(feature = "pem")]
use crate::pem;
#[cfg(feature = "pem")]
use core::str::FromStr;

/// PKCS#8 private key document
///
/// This type provides storage for a PKCS#8 private key encoded as ASN.1 DER
/// with the invariant that the contained-document is "well-formed", i.e. it
/// will parse successfully according to this crate's parsing rules.
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct PrivateKeyDocument(Zeroizing<Vec<u8>>);

impl PrivateKeyDocument {
    /// Parse [`PrivateKeyDocument`] from ASN.1 DER-encoded PKCS#8
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        PrivateKeyInfo::from_der(bytes)?;
        Ok(Self(Zeroizing::new(bytes.to_owned())))
    }

    /// Parse [`PrivateKeyDocument`] from PEM-encoded PKCS#8.
    ///
    /// PEM-encoded private keys can be identified by the leading delimiter:
    ///
    /// ```text
    /// -----BEGIN PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn from_pem(s: &str) -> Result<Self> {
        let der_bytes = pem::parse(s, pem::PRIVATE_KEY_BOUNDARY)?;
        Self::from_der(&*der_bytes)
    }

    /// Parse the [`PrivateKeyInfo`] contained in this [`PrivateKeyDocument`]
    pub fn private_key_info(&self) -> PrivateKeyInfo<'_> {
        PrivateKeyInfo::from_der(self.0.as_ref()).expect("constructor failed to validate document")
    }
}

impl AsRef<[u8]> for PrivateKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for PrivateKeyDocument {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl fmt::Debug for PrivateKeyDocument {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("PrivateKeyDocument")
            .field(&self.private_key_info())
            .finish()
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for PrivateKeyDocument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pem(s)
    }
}

/// SPKI public key document
///
/// This type provides storage for a SPKI public key encoded as ASN.1 DER with
/// the invariant that the contained-document is "well-formed", i.e. it will
/// parse successfully according to this crate's parsing rules.
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct PublicKeyDocument(Vec<u8>);

impl PublicKeyDocument {
    /// Parse [`PublicKeyDocument`] from ASN.1 DER
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        SubjectPublicKeyInfo::from_der(bytes)?;
        Ok(Self(bytes.to_owned()))
    }

    /// Parse [`PublicKeyDocument`] from PEM
    ///
    /// PEM-encoded public keys can be identified by the leading delimiter:
    ///
    /// ```text
    /// -----BEGIN PUBLIC KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn from_pem(s: &str) -> Result<Self> {
        let der_bytes = pem::parse(s, pem::PUBLIC_KEY_BOUNDARY)?;
        Self::from_der(&*der_bytes)
    }

    /// Parse the [`SubjectPublicKeyInfo`] contained in this [`PublicKeyDocument`]
    pub fn spki(&self) -> SubjectPublicKeyInfo<'_> {
        SubjectPublicKeyInfo::from_der(self.0.as_ref())
            .expect("constructor failed to validate document")
    }
}

impl AsRef<[u8]> for PublicKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for PublicKeyDocument {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl fmt::Debug for PublicKeyDocument {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("PublicKeyDocument")
            .field(&self.spki())
            .finish()
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for PublicKeyDocument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pem(s)
    }
}
