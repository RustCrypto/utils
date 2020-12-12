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
    /// Algorithm OID.
    ///
    /// This is the `algorithm` field in the `AlgorithmIdentifier` ASN.1 schema.
    pub oid: ObjectIdentifier,

    /// Algorithm parameters.
    ///
    /// According to RFC 5280, this technically contains algorithm-defined
    /// `ANY` data, however as this crate is specialized to RSA and ECC keys,
    /// we only support an OID in this field.
    pub parameters: Option<ObjectIdentifier>,
}

impl AlgorithmIdentifier {
    /// Parse [`AlgorithmIdentifier`] encoded as ASN.1 DER
    pub fn from_der(mut bytes: &[u8]) -> Result<Self> {
        let algorithm = asn1::decoder::decode_algorithm_identifier(&mut bytes)?;

        if bytes.is_empty() {
            Ok(algorithm)
        } else {
            Err(Error)
        }
    }

    /// Write an ASN.1 DER-encoded [`AlgorithmIdentifier`] to the provided
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
