//! X.509 `AlgorithmIdentifier`

use crate::{asn1, Error, ObjectIdentifier, Result};
use core::convert::TryFrom;

/// X.509 `AlgorithmIdentifier`
///
/// Defined in RFC 5280 Section 4.1.1.2:
/// <https://tools.ietf.org/html/rfc5280#section-4.1.1.2>
///
/// ```text
/// AlgorithmIdentifier  ::=  SEQUENCE  {
///      algorithm               OBJECT IDENTIFIER,
///      parameters              ANY DEFINED BY algorithm OPTIONAL  }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AlgorithmIdentifier {
    /// Algorithm OID, i.e. the `algorithm` field in the `AlgorithmIdentifier`
    /// ASN.1 schema.
    pub oid: ObjectIdentifier,

    /// Algorithm `parameters`.
    pub parameters: Option<AlgorithmParameters>,
}

impl AlgorithmIdentifier {
    /// Parse [`AlgorithmIdentifier`] encoded as ASN.1 DER
    pub fn from_der(mut bytes: &[u8]) -> Result<Self> {
        let algorithm = asn1::decoder::decode_algorithm_identifier(&mut bytes)?;

        if bytes.is_empty() {
            Ok(algorithm)
        } else {
            Err(Error::Decode)
        }
    }

    /// Get the `parameters` field as an [`ObjectIdentifier`].
    ///
    /// Returns `None` if it is absent or not an OID.
    pub fn parameters_oid(&self) -> Option<ObjectIdentifier> {
        self.parameters.and_then(AlgorithmParameters::oid)
    }

    /// Write ASN.1 DER-encoded [`AlgorithmIdentifier`] to the provided
    /// buffer, returning a slice containing the encoded data.
    pub fn write_der<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8]> {
        let offset = asn1::encoder::encode_algorithm_identifier(buffer, self)?;
        Ok(&buffer[..offset])
    }

    /// Encode this [`AlgorithmIdentifier`] as ASN.1 DER
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> alloc::vec::Vec<u8> {
        let len = asn1::encoder::algorithm_identifier_len(self).unwrap();
        let mut buffer = vec![0u8; len];
        self.write_der(&mut buffer).unwrap();
        buffer
    }
}

impl TryFrom<&[u8]> for AlgorithmIdentifier {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_der(bytes)
    }
}

/// The `parameters` field of `AlgorithmIdentifier`.
///
/// This is an algorithm-defined `ANY` field. We presently support OIDs
/// (as used by `id-ecPublicKey`) and ASN.1 `NULL` for RSA as required by
/// [RFC 3279 Section 2.3.1][1].
///
/// This enum is marked as `non_exhaustive` to potentially support other
/// algorithm-dependent data types in the future.
///
/// [1]: https://tools.ietf.org/html/rfc3279#section-2.3.1
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AlgorithmParameters {
    /// ASN.1 NULL value
    Null,

    /// [`ObjectIdentifier`] that names a sub-algorithm
    Oid(ObjectIdentifier),
}

impl AlgorithmParameters {
    /// Get the OID value if applicable
    pub fn oid(self) -> Option<ObjectIdentifier> {
        if let AlgorithmParameters::Oid(oid) = self {
            Some(oid)
        } else {
            None
        }
    }

    /// Is this parameter value NULL?
    pub fn is_null(self) -> bool {
        self == AlgorithmParameters::Null
    }

    /// Is this parameter value an OID?
    pub fn is_oid(self) -> bool {
        self.oid().is_some()
    }
}
