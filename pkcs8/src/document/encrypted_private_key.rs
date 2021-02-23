//! PKCS#8 encrypted private key document.

use crate::{EncryptedPrivateKeyInfo, Error, Result};
use alloc::{borrow::ToOwned, vec::Vec};
use core::convert::{TryFrom, TryInto};
use zeroize::{Zeroize, Zeroizing};

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
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs5")))]
pub struct EncryptedPrivateKeyDocument(Zeroizing<Vec<u8>>);

impl EncryptedPrivateKeyDocument {
    /// Parse [`EncryptedPrivateKeyDocument`] from ASN.1 DER-encoded PKCS#8.
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        bytes.try_into()
    }

    /// Load [`EncryptedPrivateKeyDocument`] from an ASN.1 DER-encoded file on the local.
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn read_der_file(path: impl AsRef<Path>) -> Result<Self> {
        fs::read(path)?.try_into()
    }

    /// Write ASN.1 DER-encoded PKCS#8 private key to the given path.
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn write_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_secret_file(path, self.as_ref())
    }

    /// Parse the [`EncryptedPrivateKeyInfo`] contained in this [`EncryptedPrivateKeyDocument`].
    pub fn encrypted_private_key_info(&self) -> EncryptedPrivateKeyInfo<'_> {
        EncryptedPrivateKeyInfo::try_from(self.0.as_ref())
            .expect("malformed EncryptedPrivateKeyDocument")
    }
}

impl AsRef<[u8]> for EncryptedPrivateKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
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
        if EncryptedPrivateKeyInfo::try_from(bytes.as_slice()).is_ok() {
            Ok(Self(Zeroizing::new(bytes)))
        } else {
            bytes.zeroize();
            Err(Error::Decode)
        }
    }
}
