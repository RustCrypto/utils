//! PKCS#8 encrypted private key document.

use crate::{error, EncryptedPrivateKeyInfo, Error, Result};
use alloc::{borrow::ToOwned, vec::Vec};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};
use der::Encodable;
use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "encryption")]
use crate::PrivateKeyDocument;

#[cfg(feature = "pem")]
use {
    crate::{encrypted_private_key_info::PEM_TYPE_LABEL, pem, LineEnding},
    alloc::string::String,
    core::str::FromStr,
};

#[cfg(feature = "std")]
use {
    super::private_key::write_secret_file,
    std::{fs, path::Path},
};

/// Encrypted PKCS#8 private key document.
///
/// This type provides heap-backed storage for [`EncryptedPrivateKeyInfo`]
/// encoded as ASN.1 DER with the invariant that the contained-document is
/// "well-formed", i.e. it will parse successfully according to this crate's
/// parsing rules.
#[derive(Clone, Eq, PartialEq)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs5")))]
pub struct EncryptedPrivateKeyDocument(Zeroizing<Vec<u8>>);

impl EncryptedPrivateKeyDocument {
    /// Attempt to decrypt this encrypted private key using the provided
    /// password to derive an encryption key.
    #[cfg(feature = "encryption")]
    #[cfg_attr(docsrs, doc(cfg(feature = "encryption")))]
    pub fn decrypt(&self, password: impl AsRef<[u8]>) -> Result<PrivateKeyDocument> {
        self.encrypted_private_key_info().decrypt(password)
    }

    /// Parse the [`EncryptedPrivateKeyInfo`] contained in this [`EncryptedPrivateKeyDocument`].
    pub fn encrypted_private_key_info(&self) -> EncryptedPrivateKeyInfo<'_> {
        EncryptedPrivateKeyInfo::try_from(self.0.as_ref())
            .expect("malformed EncryptedPrivateKeyDocument")
    }

    /// Parse [`EncryptedPrivateKeyDocument`] from ASN.1 DER-encoded PKCS#8.
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        bytes.try_into()
    }

    /// Parse [`EncryptedPrivateKeyDocument`] from PEM-encoded PKCS#8.
    ///
    /// PEM-encoded encrypted private keys can be identified by the leading
    /// delimiter:
    ///
    /// ```text
    /// -----BEGIN ENCRYPTED PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn from_pem(s: &str) -> Result<Self> {
        let (label, der_bytes) = pem::decode_vec(s.as_bytes())?;

        if label != PEM_TYPE_LABEL {
            return Err(pem::Error::Label.into());
        }

        Self::from_der(&*der_bytes)
    }

    /// Serialize [`EncryptedPrivateKeyDocument`] as self-zeroizing PEM-encoded
    /// PKCS#8 string.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> Zeroizing<String> {
        self.to_pem_with_le(LineEnding::default())
    }

    /// Serialize [`EncryptedPrivateKeyDocument`] as self-zeroizing PEM-encoded
    /// PKCS#8 string with the given [`LineEnding`].
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem_with_le(&self, line_ending: LineEnding) -> Zeroizing<String> {
        Zeroizing::new(
            pem::encode_string(PEM_TYPE_LABEL, line_ending, &self.0)
                .expect(error::PEM_ENCODING_MSG),
        )
    }

    /// Load [`EncryptedPrivateKeyDocument`] from an ASN.1 DER-encoded file on
    /// the local filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn read_der_file(path: impl AsRef<Path>) -> Result<Self> {
        fs::read(path)?.try_into()
    }

    /// Load [`EncryptedPrivateKeyDocument`] from a PEM-encoded file on the
    /// local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn read_pem_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_pem(&Zeroizing::new(fs::read_to_string(path)?))
    }

    /// Write ASN.1 DER-encoded PKCS#8 encrypted private key to the given path.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn write_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_secret_file(path, self.as_ref())
    }

    /// Write PEM-encoded PKCS#8 encrypted private key to the given path.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn write_pem_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_secret_file(path, self.to_pem().as_bytes())
    }
}

impl AsRef<[u8]> for EncryptedPrivateKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<EncryptedPrivateKeyInfo<'_>> for EncryptedPrivateKeyDocument {
    fn from(key: EncryptedPrivateKeyInfo<'_>) -> EncryptedPrivateKeyDocument {
        EncryptedPrivateKeyDocument::from(&key)
    }
}

impl From<&EncryptedPrivateKeyInfo<'_>> for EncryptedPrivateKeyDocument {
    fn from(key: &EncryptedPrivateKeyInfo<'_>) -> EncryptedPrivateKeyDocument {
        key.to_vec()
            .ok()
            .and_then(|buf| buf.try_into().ok())
            .expect(error::DER_ENCODING_MSG)
    }
}

impl TryFrom<&[u8]> for EncryptedPrivateKeyDocument {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        EncryptedPrivateKeyInfo::try_from(bytes)?;
        Ok(Self(Zeroizing::new(bytes.to_owned())))
    }
}

impl TryFrom<Vec<u8>> for EncryptedPrivateKeyDocument {
    type Error = Error;

    fn try_from(mut bytes: Vec<u8>) -> Result<Self> {
        // Ensure document is well-formed
        if let Err(err) = EncryptedPrivateKeyInfo::try_from(bytes.as_slice()) {
            bytes.zeroize();
            return Err(err);
        }

        Ok(Self(Zeroizing::new(bytes)))
    }
}

impl fmt::Debug for EncryptedPrivateKeyDocument {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("EncryptedPrivateKeyDocument")
            .field(&self.encrypted_private_key_info())
            .finish()
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for EncryptedPrivateKeyDocument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pem(s)
    }
}
