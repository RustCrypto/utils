//! X.509 `AlgorithmIdentifier`

use crate::{Error, ObjectIdentifier, Result};
use core::convert::TryFrom;
use der::Decodable;

#[cfg(feature = "alloc")]
use crate::algorithm;

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
    pub fn from_der(bytes: &[u8]) -> Result<Self> {
        let mut decoder = der::Decoder::new(bytes);
        let result = Self::decode(&mut decoder)?;
        decoder.finish(result).map_err(|_| Error::Decode)
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
        let offset = encode_identifier(buffer, self)?;
        Ok(&buffer[..offset])
    }

    /// Encode this [`AlgorithmIdentifier`] as ASN.1 DER
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_der(&self) -> alloc::vec::Vec<u8> {
        let len = algorithm::identifier_len(self).unwrap();
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

impl TryFrom<der::Any<'_>> for AlgorithmIdentifier {
    type Error = der::Error;

    fn try_from(any: der::Any<'_>) -> der::Result<AlgorithmIdentifier> {
        any.sequence(|mut decoder| {
            let oid = decoder.oid()?;
            let parameters = decoder.optional()?;
            decoder.finish(Self { oid, parameters })
        })
    }
}

impl der::Tagged for AlgorithmIdentifier {
    const TAG: der::Tag = der::Tag::Sequence;
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

impl From<der::Null> for AlgorithmParameters {
    fn from(_: der::Null) -> AlgorithmParameters {
        AlgorithmParameters::Null
    }
}

impl From<ObjectIdentifier> for AlgorithmParameters {
    fn from(oid: ObjectIdentifier) -> AlgorithmParameters {
        AlgorithmParameters::Oid(oid)
    }
}

impl TryFrom<der::Any<'_>> for AlgorithmParameters {
    type Error = der::Error;

    fn try_from(any: der::Any<'_>) -> der::Result<AlgorithmParameters> {
        match any.tag() {
            der::Tag::Null => any.null().map(Into::into),
            der::Tag::ObjectIdentifier => any.oid().map(Into::into),
            _ => Err(der::Error::UnexpectedTag {
                expected: None,
                actual: any.tag(),
            }),
        }
    }
}

/// Encode an [`AlgorithmIdentifier`].
pub(crate) fn encode_identifier(
    buffer: &mut [u8],
    algorithm_id: &AlgorithmIdentifier,
) -> Result<usize> {
    let alg_oid_len = der::length::oid(algorithm_id.oid)?;
    let params_len = parameters_len(algorithm_id)?;
    let sequence_len = alg_oid_len.checked_add(params_len).unwrap();

    let mut offset = der::encode::header(buffer, der::Tag::Sequence, sequence_len)?;
    offset += der::encode::oid(&mut buffer[offset..], algorithm_id.oid)?;
    offset += match algorithm_id.parameters {
        Some(AlgorithmParameters::Null) => {
            der::encode::header(&mut buffer[offset..], der::Tag::Null, 0)?
        }
        Some(AlgorithmParameters::Oid(oid)) => der::encode::oid(&mut buffer[offset..], oid)?,
        None => 0,
    };

    Ok(offset)
}

/// Get the length of a DER-encoded [`AlgorithmIdentifier`]
pub(crate) fn identifier_len(algorithm_id: &AlgorithmIdentifier) -> Result<usize> {
    let alg_oid_len = der::length::oid(algorithm_id.oid)?;
    let params_len = parameters_len(algorithm_id)?;
    let sequence_len = alg_oid_len.checked_add(params_len).unwrap();

    der::length::header(sequence_len)
        .ok()
        .and_then(|n| n.checked_add(sequence_len))
        .ok_or(Error::Encode)
}

/// Get the length of the `parameters` field of a DER-encoded
/// [`AlgorithmIdentifier`].
fn parameters_len(algorithm_id: &AlgorithmIdentifier) -> Result<usize> {
    match algorithm_id.parameters {
        Some(AlgorithmParameters::Null) => Ok(2),
        Some(AlgorithmParameters::Oid(oid)) => Ok(der::length::oid(oid)?),
        None => Ok(0),
    }
}
