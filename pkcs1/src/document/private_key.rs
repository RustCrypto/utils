//! PKCS#1 RSA private key document.

use crate::{error, Error, Result, RsaPrivateKey};
use alloc::{borrow::ToOwned, vec::Vec};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};
use der::Encodable;
use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "pem")]
use {
    crate::{pem, private_key::PEM_TYPE_LABEL},
    alloc::string::String,
    core::str::FromStr,
};

#[cfg(feature = "std")]
use std::{fs, path::Path, str};

/// PKCS#1 `RSA PRIVATE KEY` document.
///
/// This type provides storage for [`RsaPrivateKey`] encoded as ASN.1 DER
/// with the invariant that the contained-document is "well-formed", i.e. it
/// will parse successfully according to this crate's parsing rules.
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct RsaPrivateKeyDocument(Zeroizing<Vec<u8>>);

impl RsaPrivateKeyDocument {
    /// Parse the [`RsaPrivateKey`] contained in this [`RsaPrivateKeyDocument`]
    pub fn private_key(&self) -> RsaPrivateKey<'_> {
        RsaPrivateKey::try_from(self.0.as_ref()).expect("malformed PrivateKeyDocument")
    }

    /// Parse [`RsaPrivateKeyDocument`] from ASN.1 DER.
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        bytes.try_into()
    }

    /// Parse [`RsaPrivateKeyDocument`] from PEM.
    ///
    /// PEM-encoded private keys can be identified by the leading delimiter:
    ///
    /// ```text
    /// -----BEGIN RSA PRIVATE KEY-----
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

    /// Serialize [`RsaPrivateKeyDocument`] as self-zeroizing PEM-encoded PKCS#1 RSA string.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> Zeroizing<String> {
        Zeroizing::new(pem::encode_string(PEM_TYPE_LABEL, &self.0).expect(error::PEM_ENCODING_MSG))
    }

    /// Load [`RsaPrivateKeyDocument`] from an ASN.1 DER-encoded file on the local
    /// filesystem (binary format).
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn read_der_file(path: impl AsRef<Path>) -> Result<Self> {
        fs::read(path)?.try_into()
    }

    /// Load [`RsaPrivateKeyDocument`] from a PEM-encoded file on the local filesystem.
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn read_pem_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_pem(&Zeroizing::new(fs::read_to_string(path)?))
    }

    /// Write ASN.1 DER-encoded PKCS#1 RSA private key to the given path
    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn write_der_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_secret_file(path, self.as_ref())
    }

    /// Write PEM-encoded PKCS#1 RSA private key to the given path
    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    pub fn write_pem_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_secret_file(path, self.to_pem().as_bytes())
    }
}

impl AsRef<[u8]> for RsaPrivateKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<RsaPrivateKey<'_>> for RsaPrivateKeyDocument {
    fn from(private_key: RsaPrivateKey<'_>) -> RsaPrivateKeyDocument {
        RsaPrivateKeyDocument::from(&private_key)
    }
}

impl From<&RsaPrivateKey<'_>> for RsaPrivateKeyDocument {
    fn from(private_key: &RsaPrivateKey<'_>) -> RsaPrivateKeyDocument {
        private_key
            .to_vec()
            .ok()
            .and_then(|buf| buf.try_into().ok())
            .expect(error::DER_ENCODING_MSG)
    }
}

impl TryFrom<&[u8]> for RsaPrivateKeyDocument {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        RsaPrivateKey::try_from(bytes)?;
        Ok(Self(Zeroizing::new(bytes.to_owned())))
    }
}

impl TryFrom<Vec<u8>> for RsaPrivateKeyDocument {
    type Error = Error;

    fn try_from(mut bytes: Vec<u8>) -> Result<Self> {
        // Ensure document is well-formed
        if let Err(err) = RsaPrivateKey::try_from(bytes.as_slice()) {
            bytes.zeroize();
            return Err(err.into());
        }

        Ok(Self(Zeroizing::new(bytes)))
    }
}

impl fmt::Debug for RsaPrivateKeyDocument {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("RsaPrivateKeyDocument")
            .field(&self.private_key())
            .finish()
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for RsaPrivateKeyDocument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pem(s)
    }
}

/// Write a file containing secret data to the filesystem, restricting the
/// file permissions so it's only readable by the owner
#[cfg(all(unix, feature = "std"))]
pub(super) fn write_secret_file(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};

    /// File permissions for secret data
    #[cfg(unix)]
    const SECRET_FILE_PERMS: u32 = 0o600;

    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(SECRET_FILE_PERMS)
        .open(path)
        .and_then(|mut file| file.write_all(data))?;

    Ok(())
}

/// Write a file containing secret data to the filesystem
// TODO(tarcieri): permissions hardening on Windows
#[cfg(all(not(unix), feature = "std"))]
pub(super) fn write_secret_file(path: impl AsRef<Path>, data: &[u8]) -> Result<()> {
    fs::write(path, data)?;
    Ok(())
}
