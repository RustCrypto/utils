//! PKCS#8 `PrivateKeyInfo`.

use crate::{asn1, AlgorithmIdentifier, Error, Result};
use core::{convert::TryFrom, fmt};

#[cfg(feature = "alloc")]
use crate::document::PrivateKeyDocument;
#[cfg(feature = "alloc")]
use zeroize::Zeroizing;

#[cfg(feature = "pem")]
use crate::pem;

/// PKCS#8 `PrivateKeyInfo`
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`] and private key
/// data in an algorithm specific format.
///
/// Described in RFC 5208 Section 5:
/// <https://tools.ietf.org/html/rfc5208#section-5>
///
/// ```text
/// PrivateKeyInfo ::= SEQUENCE {
///         version                   Version,
///         privateKeyAlgorithm       PrivateKeyAlgorithmIdentifier,
///         privateKey                PrivateKey,
///         attributes           [0]  IMPLICIT Attributes OPTIONAL }
///
/// Version ::= INTEGER
///
/// PrivateKeyAlgorithmIdentifier ::= AlgorithmIdentifier
///
/// PrivateKey ::= OCTET STRING
///
/// Attributes ::= SET OF Attribute
/// ```
#[derive(Copy, Clone)]
pub struct PrivateKeyInfo<'a> {
    /// X.509 [`AlgorithmIdentifier`]
    pub algorithm: AlgorithmIdentifier,

    /// Private key data
    pub private_key: &'a [u8],
}

impl<'a> PrivateKeyInfo<'a> {
    /// Parse [`PrivateKeyInfo`] encoded as ASN.1 DER.
    pub fn from_der(bytes: &'a [u8]) -> Result<Self> {
        asn1::decoder::decode_private_key_info(bytes)
    }

    /// Write ASN.1 DER-encoded [`PrivateKeyInfo`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'b>(&self, buffer: &'b mut [u8]) -> Result<&'b [u8]> {
        let offset = asn1::encoder::encode_private_key_info(buffer, self)?;
        Ok(&buffer[..offset])
    }

    /// Encode this [`PrivateKeyInfo`] as ASN.1 DER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> PrivateKeyDocument {
        let len = asn1::encoder::private_key_info_len(self).unwrap();
        let mut buffer = Zeroizing::new(vec![0u8; len]);
        self.write_der(&mut buffer).unwrap();
        PrivateKeyDocument::from_der(&buffer).expect("malformed DER")
    }

    /// Encode this [`PrivateKeyInfo`] as PEM-encoded ASN.1 DER.
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    pub fn to_pem(&self) -> Zeroizing<alloc::string::String> {
        let doc = self.to_der();
        let pem = pem::serialize(doc.as_ref(), pem::PRIVATE_KEY_BOUNDARY).expect("malformed PEM");
        Zeroizing::new(pem)
    }
}

impl<'a> TryFrom<&'a [u8]> for PrivateKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

impl<'a> fmt::Debug for PrivateKeyInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKeyInfo")
            .field("algorithm", &self.algorithm)
            .finish() // TODO(tarcieri): use `finish_non_exhaustive` when stable
    }
}
