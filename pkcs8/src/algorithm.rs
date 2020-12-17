//! X.509 `AlgorithmIdentifier`

use crate::{Error, ObjectIdentifier, Result};
use core::convert::TryFrom;

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
    pub fn from_der(mut bytes: &[u8]) -> Result<Self> {
        let algorithm_id = decode_identifier(&mut bytes)?;

        if bytes.is_empty() {
            Ok(algorithm_id)
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

/// Decode [`AlgorithmIdentifier`] from ASN.1 DER
pub(crate) fn decode_identifier(input: &mut &[u8]) -> Result<AlgorithmIdentifier> {
    let mut bytes = der::decode::nested(input, der::Tag::Sequence)?;

    // Check OBJECT ID header
    // TODO(tarcieri): use `der::decode::oid`
    if der::decode::byte(&mut bytes)? != der::Tag::ObjectIdentifier as u8 {
        return Err(Error::Decode);
    }

    let len = der::decode::length(&mut bytes)?;

    if len > bytes.len() {
        return Err(Error::Decode);
    }

    let (alg_bytes, mut param_bytes) = bytes.split_at(len);
    let algorithm = ObjectIdentifier::from_ber(alg_bytes).map_err(|_| Error::Decode)?;

    let parameters = if param_bytes.is_empty() {
        None
    } else {
        let tag = der::decode::byte(&mut param_bytes)?;

        if tag == der::Tag::Null as u8 {
            if der::decode::length(&mut param_bytes)? != 0 {
                return Err(Error::Decode);
            }

            // Disallow any trailing data after the parameters
            if !param_bytes.is_empty() {
                return Err(Error::Decode);
            }

            Some(AlgorithmParameters::Null)
        } else if tag == der::Tag::ObjectIdentifier as u8 {
            // TODO(tarcieri): use `der::decode::oid`
            let len = der::decode::length(&mut param_bytes)?;

            if len != param_bytes.len() {
                return Err(Error::Decode);
            }

            let oid = ObjectIdentifier::from_ber(param_bytes).map_err(|_| Error::Decode)?;
            Some(AlgorithmParameters::Oid(oid))
        } else {
            return Err(Error::Decode);
        }
    };

    Ok(AlgorithmIdentifier {
        oid: algorithm,
        parameters,
    })
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
