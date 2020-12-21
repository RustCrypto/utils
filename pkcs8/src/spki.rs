//! X.509 `SubjectPublicKeyInfo`

use crate::{AlgorithmIdentifier, Error, Result};
use core::convert::TryFrom;
use der::{Decodable, Encodable, Message};

#[cfg(feature = "alloc")]
use {
    crate::{error, PublicKeyDocument},
    core::convert::TryInto,
};

#[cfg(feature = "pem")]
use crate::pem;

/// X.509 `SubjectPublicKeyInfo` (SPKI)
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`] and public key
/// data in an algorithm specific format.
///
/// Described in RFC 5208 Section 4.1:
/// <https://tools.ietf.org/html/rfc5280#section-4.1>
///
/// ```text
///    SubjectPublicKeyInfo  ::=  SEQUENCE  {
///         algorithm            AlgorithmIdentifier,
///         subjectPublicKey     BIT STRING  }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SubjectPublicKeyInfo<'a> {
    /// X.509 [`AlgorithmIdentifier`]
    pub algorithm: AlgorithmIdentifier,

    /// Public key data
    pub subject_public_key: &'a [u8],
}

impl<'a> SubjectPublicKeyInfo<'a> {
    /// Parse [`SubjectPublicKeyInfo`] encoded as ASN.1 DER.
    pub fn from_der(bytes: &'a [u8]) -> Result<Self> {
        Ok(Self::from_bytes(bytes)?)
    }

    /// Write ASN.1 DER-encoded [`SubjectPublicKeyInfo`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'b>(&self, buffer: &'b mut [u8]) -> Result<&'b [u8]> {
        Ok(self.encode_to_slice(buffer)?)
    }

    /// Encode this [`SubjectPublicKeyInfo] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PublicKeyDocument {
        self.to_vec()
            .ok()
            .and_then(|buf| buf.try_into().ok())
            .expect(error::DER_ENCODING_MSG)
    }

    /// Encode this [`SubjectPublicKeyInfo`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> alloc::string::String {
        let doc = self.to_der();
        pem::encode(doc.as_ref(), pem::PUBLIC_KEY_BOUNDARY)
    }
}

impl<'a> TryFrom<&'a [u8]> for SubjectPublicKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl<'a> TryFrom<der::Any<'a>> for SubjectPublicKeyInfo<'a> {
    type Error = der::Error;

    fn try_from(any: der::Any<'a>) -> der::Result<SubjectPublicKeyInfo<'a>> {
        any.sequence(|decoder| {
            let algorithm = decoder.decode()?;
            let subject_public_key = decoder.bit_string()?.as_bytes();
            Ok(Self {
                algorithm,
                subject_public_key,
            })
        })
    }
}

impl<'a> Message<'a> for SubjectPublicKeyInfo<'a> {
    fn fields<F, T>(&self, f: F) -> der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> der::Result<T>,
    {
        f(&[
            &self.algorithm,
            &der::BitString::new(self.subject_public_key)?,
        ])
    }
}
