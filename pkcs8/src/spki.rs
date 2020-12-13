//! X.509 `SubjectPublicKeyInfo`

use crate::{asn1, AlgorithmIdentifier, Error, Result};
use core::convert::TryFrom;

#[cfg(feature = "alloc")]
use {crate::PublicKeyDocument, core::convert::TryInto};

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
        asn1::decoder::decode_spki(bytes)
    }

    /// Write ASN.1 DER-encoded [`SubjectPublicKeyInfo`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'b>(&self, buffer: &'b mut [u8]) -> Result<&'b [u8]> {
        let offset = asn1::encoder::encode_spki(buffer, self)?;
        Ok(&buffer[..offset])
    }

    /// Encode this [`SubjectPublicKeyInfo] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PublicKeyDocument {
        let len = asn1::encoder::spki_len(self).unwrap();
        let mut buffer = vec![0u8; len];
        self.write_der(&mut buffer).unwrap();
        buffer.try_into().expect("malformed DER")
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
