//! PKCS#1 RSA public key document.

use crate::{error, Error, FromRsaPublicKey, Result, RsaPublicKey, ToRsaPublicKey};
use alloc::{borrow::ToOwned, vec::Vec};
use core::{
    convert::{TryFrom, TryInto},
    fmt,
};
use der::Encodable;

#[cfg(feature = "std")]
use std::{fs, path::Path, str};

#[cfg(feature = "pem")]
use {
    crate::{pem, public_key::PEM_TYPE_LABEL, LineEnding},
    alloc::string::String,
    core::str::FromStr,
};

/// PKCS#1 `RSA PUBLIC KEY` document.
///
/// This type provides storage for [`RsaPublicKey`] encoded as ASN.1
/// DER with the invariant that the contained-document is "well-formed", i.e.
/// it will parse successfully according to this crate's parsing rules.
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct RsaPublicKeyDocument(Vec<u8>);

impl RsaPublicKeyDocument {
    /// Parse the [`RsaPublicKey`] contained in this [`RsaPublicKeyDocument`]
    pub fn public_key(&self) -> RsaPublicKey<'_> {
        RsaPublicKey::try_from(self.0.as_slice()).expect("malformed PublicKeyDocument")
    }

    /// Borrow the inner DER encoded bytes.
    pub fn as_der(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl FromRsaPublicKey for RsaPublicKeyDocument {
    fn from_pkcs1_public_key(public_key: RsaPublicKey<'_>) -> Result<Self> {
        Ok(Self(public_key.to_vec()?))
    }

    fn from_pkcs1_der(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        RsaPublicKey::try_from(bytes)?;
        Ok(Self(bytes.to_owned()))
    }

    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs1_pem(s: &str) -> Result<Self> {
        let (label, der_bytes) = pem::decode_vec(s.as_bytes())?;

        if label != PEM_TYPE_LABEL {
            return Err(pem::Error::Label.into());
        }

        // Ensure document is well-formed
        RsaPublicKey::try_from(der_bytes.as_slice())?;
        Ok(Self(der_bytes))
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
        Self::from_pkcs1_pem(&fs::read_to_string(path)?)
    }
}

impl ToRsaPublicKey for RsaPublicKeyDocument {
    fn to_pkcs1_der(&self) -> Result<RsaPublicKeyDocument> {
        Ok(self.clone())
    }

    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn to_pkcs1_pem_with_le(&self, line_ending: LineEnding) -> Result<String> {
        Ok(pem::encode_string(PEM_TYPE_LABEL, line_ending, &self.0)?)
    }

    #[cfg(feature = "std")]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_der_file(&self, path: &Path) -> Result<()> {
        fs::write(path, self.as_ref())?;
        Ok(())
    }

    #[cfg(all(feature = "pem", feature = "std"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
    fn write_pkcs1_pem_file(&self, path: &Path) -> Result<()> {
        fs::write(path, self.to_pkcs1_pem()?.as_bytes())?;
        Ok(())
    }
}

impl AsRef<[u8]> for RsaPublicKeyDocument {
    fn as_ref(&self) -> &[u8] {
        self.as_der()
    }
}

impl From<RsaPublicKey<'_>> for RsaPublicKeyDocument {
    fn from(public_key: RsaPublicKey<'_>) -> RsaPublicKeyDocument {
        RsaPublicKeyDocument::from(&public_key)
    }
}

impl From<&RsaPublicKey<'_>> for RsaPublicKeyDocument {
    fn from(public_key: &RsaPublicKey<'_>) -> RsaPublicKeyDocument {
        public_key
            .to_vec()
            .ok()
            .and_then(|buf| buf.try_into().ok())
            .expect(error::DER_ENCODING_MSG)
    }
}

impl TryFrom<&[u8]> for RsaPublicKeyDocument {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        RsaPublicKeyDocument::from_pkcs1_der(bytes)
    }
}

impl TryFrom<Vec<u8>> for RsaPublicKeyDocument {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        // Ensure document is well-formed
        RsaPublicKey::try_from(bytes.as_slice())?;
        Ok(Self(bytes))
    }
}

impl fmt::Debug for RsaPublicKeyDocument {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_tuple("RsaPublicKeyDocument")
            .field(&self.public_key())
            .finish()
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for RsaPublicKeyDocument {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pkcs1_pem(s)
    }
}
