//! X.509 `AlgorithmIdentifier`

use crate::{asn1, Error, ObjectIdentifier, Result};

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
    pub algorithm: ObjectIdentifier,

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
        let algorithm = asn1::parse_algorithm_identifier(&mut bytes)?;

        if bytes.is_empty() {
            Ok(algorithm)
        } else {
            Err(Error)
        }
    }
}
