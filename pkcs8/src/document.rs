//! PKCS#8 documents: serialized PKCS#8 private keys
// TODO(tarcieri): heapless support?

use crate::{Error, PrivateKeyInfo, Result};
use alloc::{borrow::ToOwned, vec::Vec};
use core::{convert::TryFrom, fmt};
use zeroize::Zeroizing;

#[cfg(feature = "pem")]
use crate::pem;
#[cfg(feature = "pem")]
use core::str::FromStr;

/// PKCS#8 document
///
/// This type provides heapless storage for a PKCS#8 encoded private key with
/// the invariant that the contained-document is "well-formed", i.e. it will
/// parse successfully according to this crate's parsing rules.
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub struct Document(Zeroizing<Vec<u8>>);

impl Document {
    /// Parse [`Document`] from ASN.1 DER-encoded PKCS#8
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        // Ensure document is well-formed
        PrivateKeyInfo::from_der(bytes)?;
        Ok(Self(Zeroizing::new(bytes.to_owned())))
    }

    /// Parse [`Document`] from PEM-encoded PKCS#8.
    ///
    /// PEM-encoding can be identified by the leading delimiter:
    ///
    /// ```text
    /// -----BEGIN PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn from_pem(s: &str) -> Result<Self> {
        pem::parse(s)
    }

    /// Parse the [`PrivateKeyInfo`] contained in this [`Document`]
    pub fn private_key_info(&self) -> PrivateKeyInfo<'_> {
        PrivateKeyInfo::from_der(self.0.as_ref()).expect("constructor failed to validate document")
    }
}

impl AsRef<[u8]> for Document {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for Document {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Document(...)")
    }
}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for Document {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pem(s)
    }
}
