//! PKCS#8 `PrivateKeyInfo`.

use crate::{asn1, AlgorithmIdentifier, Error, Result};
use core::{convert::TryFrom, fmt};

/// PKCS#8 `PrivateKeyInfo`.
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
    /// Parse [`PrivateKeyInfo`] encoded as ASN.1 DER
    pub fn from_der(bytes: &'a [u8]) -> Result<Self> {
        asn1::parse_private_key_info(bytes)
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
