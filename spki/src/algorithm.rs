//! X.509 `AlgorithmIdentifier`

use core::convert::TryFrom;
use der::{Decodable, Encodable, Error, Message, Null, ObjectIdentifier, Result, Tag};

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
    /// Get the `parameters` field as an [`ObjectIdentifier`].
    ///
    /// Returns `None` if it is absent or not an OID.
    pub fn parameters_oid(&self) -> Option<ObjectIdentifier> {
        self.parameters.and_then(AlgorithmParameters::oid)
    }
}

impl TryFrom<&[u8]> for AlgorithmIdentifier {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_bytes(bytes)
    }
}

impl TryFrom<der::Any<'_>> for AlgorithmIdentifier {
    type Error = Error;

    fn try_from(any: der::Any<'_>) -> Result<AlgorithmIdentifier> {
        any.sequence(|decoder| {
            let oid = decoder.decode()?;
            let parameters = decoder.decode()?;
            Ok(Self { oid, parameters })
        })
    }
}

impl<'a> Message<'a> for AlgorithmIdentifier {
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        f(&[&self.oid, &self.parameters])
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

impl From<Null> for AlgorithmParameters {
    fn from(_: Null) -> AlgorithmParameters {
        AlgorithmParameters::Null
    }
}

impl From<ObjectIdentifier> for AlgorithmParameters {
    fn from(oid: ObjectIdentifier) -> AlgorithmParameters {
        AlgorithmParameters::Oid(oid)
    }
}

impl TryFrom<der::Any<'_>> for AlgorithmParameters {
    type Error = Error;

    fn try_from(any: der::Any<'_>) -> Result<AlgorithmParameters> {
        match any.tag() {
            Tag::Null => any.null().map(Into::into),
            Tag::ObjectIdentifier => any.oid().map(Into::into),
            _ => Err(der::ErrorKind::UnexpectedTag {
                expected: None,
                actual: any.tag(),
            }
            .into()),
        }
    }
}

impl Encodable for AlgorithmParameters {
    fn encoded_len(&self) -> Result<der::Length> {
        match self {
            Self::Null => Null.encoded_len(),
            Self::Oid(oid) => oid.encoded_len(),
        }
    }

    fn encode(&self, encoder: &mut der::Encoder<'_>) -> Result<()> {
        match self {
            Self::Null => encoder.null(),
            Self::Oid(oid) => encoder.oid(*oid),
        }
    }
}
