//! PKCS#1 RSA private key document.

use crate::{error, Error, FromRsaPrivateKey, Result, RsaPrivateKey, ToRsaPrivateKey};
use alloc::{borrow::ToOwned, vec::Vec};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};
use der::Encodable;
use zeroize::{Zeroize, Zeroizing};

#[cfg(feature = "pem")]
use {
    crate::{pem, private_key::PEM_TYPE_LABEL, LineEnding},
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

    /// Borrow the inner DER encoded bytes.
    pub fn as_der(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl FromRsaPrivateKey for RsaPrivateKeyDocument {
    fn from_pkcs1_private_key(private_key: RsaPrivateKey<'_>) -> Result<Self> {
        Ok(Self(Zeroizing::new(private_key.to_vec()?)))
    }

    fn from_pkcs1_der(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        RsaPrivateKey::try_from(bytes)?;
        Ok(Self(Zeroizing::new(bytes.to_owned())))
    }

    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs1_pem(s: &str) -> Result<Self> {
        let (label, der_bytes) = pem::decode_vec(s.as_bytes())?;

        if label != PEM_TYPE_LABEL {
            return Err(pem::Error::Label.into());
        }

        // Ensure document is well-formed
        RsaPrivateKey::try_from(der_bytes.as_slice())?;
        Ok(Self(Zeroizing::new(der_bytes)))
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs1_der_file(path: &Path) -> Result<Self> {
        fs::read(path)?.try_into()
    }

    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn read_pkcs1_pem_file(path: &Path) -> Result<Self> {
        Self::from_pkcs1_pem(&Zeroizing::new(fs::read_to_string(path)?))
    }
}

impl ToRsaPrivateKey for RsaPrivateKeyDocument {
    fn to_pkcs1_der(&self) -> Result<RsaPrivateKeyDocument> {
        Ok(self.clone())
    }

    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs1_pem_with_le(&self, line_ending: LineEnding) -> Result<Zeroizing<String>> {
        let pem_doc = pem::encode_string(PEM_TYPE_LABEL, line_ending, self.as_der())?;
        Ok(Zeroizing::new(pem_doc))
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_der_file(&self, path: &Path) -> Result<()> {
        write_secret_file(path, self.as_der())
    }

    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_pem_file(&self, path: &Path) -> Result<()> {
        let pem_doc = self.to_pkcs1_pem()?;
        write_secret_file(path, pem_doc.as_bytes())
    }
}

impl AsRef<[u8]> for RsaPrivateKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.as_der()
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
        RsaPrivateKeyDocument::from_pkcs1_der(bytes)
    }
}

impl TryFrom<Vec<u8>> for RsaPrivateKeyDocument {
    type Error = Error;

    fn try_from(mut bytes: Vec<u8>) -> Result<Self> {
        // Ensure document is well-formed
        if let Err(err) = RsaPrivateKey::try_from(bytes.as_slice()) {
            bytes.zeroize();
            return Err(err);
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
        Self::from_pkcs1_pem(s)
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
