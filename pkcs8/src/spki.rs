//! X.509 `SubjectPublicKeyInfo`

use crate::{asn1, AlgorithmIdentifier, Error, Result};
use core::convert::TryFrom;

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

    /// Public key data. This is technically an ASN.1 `BIT STRING`, but this
    /// implementation constrains its contents to an octet granularity
    pub subject_public_key: &'a [u8],
}

impl<'a> SubjectPublicKeyInfo<'a> {
    /// Parse [`SubjectPublicKeyInfo`] encoded as ASN.1 DER
    pub fn from_der(bytes: &'a [u8]) -> Result<Self> {
        asn1::parse_spki(bytes)
    }
}

impl<'a> TryFrom<&'a [u8]> for SubjectPublicKeyInfo<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}
