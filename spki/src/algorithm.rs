//! X.509 `AlgorithmIdentifier`

use core::convert::TryFrom;
use der::{
    Any, Decodable, Encodable, Encoder, Error, Length, Message, Null, ObjectIdentifier, Result, Tag,
};

/// X.509 `AlgorithmIdentifier` as defined in [RFC 5280 Section 4.1.1.2].
///
/// ```text
/// AlgorithmIdentifier  ::=  SEQUENCE  {
///      algorithm               OBJECT IDENTIFIER,
///      parameters              ANY DEFINED BY algorithm OPTIONAL  }
/// ```
///
/// [RFC 5280 Section 4.1.1.2]: https://tools.ietf.org/html/rfc5280#section-4.1.1.2
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AlgorithmIdentifier<'a> {
    /// Algorithm OID, i.e. the `algorithm` field in the `AlgorithmIdentifier`
    /// ASN.1 schema.
    pub oid: ObjectIdentifier,

    /// Algorithm `parameters`.
    pub parameters: Option<AlgorithmParameters<'a>>,
}

impl<'a> AlgorithmIdentifier<'a> {
    /// Get the `parameters` field as an [`Any`].
    ///
    /// Returns an error if `parameters` are `None`, or if they are `Some`
    /// but are an [`ObjectIdentifier`] or [`Null`], i.e. this method is
    /// explicitly for handling cases other than those two.
    pub fn parameters_any(&self) -> Result<Any<'a>> {
        let params = self.parameters.ok_or(der::ErrorKind::Truncated)?;

        params.any().ok_or_else(|| {
            der::ErrorKind::UnexpectedTag {
                expected: Some(der::Tag::Sequence),
                actual: params.tag(),
            }
            .into()
        })
    }

    /// Get the `parameters` field as an [`ObjectIdentifier`].
    ///
    /// Returns an error if it is absent or not an OID.
    pub fn parameters_oid(&self) -> Result<ObjectIdentifier> {
        let params = self.parameters.ok_or(der::ErrorKind::Truncated)?;

        params.oid().ok_or_else(|| {
            der::ErrorKind::UnexpectedTag {
                expected: Some(der::Tag::Sequence),
                actual: params.tag(),
            }
            .into()
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for AlgorithmIdentifier<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self> {
        Self::from_bytes(bytes)
    }
}

impl<'a> TryFrom<Any<'a>> for AlgorithmIdentifier<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<AlgorithmIdentifier<'a>> {
        any.sequence(|decoder| {
            let oid = decoder.decode()?;
            let parameters = decoder.decode()?;
            Ok(Self { oid, parameters })
        })
    }
}

impl<'a> Message<'a> for AlgorithmIdentifier<'a> {
    fn fields<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        f(&[&self.oid, &self.parameters])
    }
}

/// The `parameters` field of `AlgorithmIdentifier`.
///
/// This is an algorithm-defined `ANY` field, but we map it into an `enum`
/// for now to simplify the [`ObjectIdentifier`] use case.
///
/// Ideally this type can eventually go away and be replaced by [`Any`]
/// with the assistance of OID reference types. See the following tracking
/// issue for more info:
///
/// <https://github.com/RustCrypto/utils/issues/266>
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AlgorithmParameters<'a> {
    /// Catch-all ASN.1 `ANY` type.
    ///
    /// Types which don't map to the other variants of this enum will be mapped
    /// to this one instead.
    Any(Any<'a>),

    /// ASN.1 `NULL` value
    Null,

    /// [`ObjectIdentifier`] that names a specific algorithm within a larger
    /// algorithm family.
    Oid(ObjectIdentifier),
}

impl<'a> AlgorithmParameters<'a> {
    /// Get the [`Any`] value if applicable.
    ///
    /// Note that this will return [`None`] in the event the parameter is an
    /// OID or `NULL`.
    pub fn any(self) -> Option<Any<'a>> {
        match self {
            AlgorithmParameters::Any(any) => Some(any),
            _ => None,
        }
    }

    /// Get the OID value if applicable
    pub fn oid(self) -> Option<ObjectIdentifier> {
        match self {
            AlgorithmParameters::Oid(oid) => Some(oid),
            _ => None,
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

    /// Get the ASN.1 DER [`Tag`] for these parameters
    pub fn tag(self) -> Tag {
        match self {
            Self::Any(any) => any.tag(),
            Self::Null => Tag::Null,
            Self::Oid(_) => Tag::ObjectIdentifier,
        }
    }
}

impl<'a> From<Null> for AlgorithmParameters<'a> {
    fn from(_: Null) -> AlgorithmParameters<'a> {
        AlgorithmParameters::Null
    }
}

impl<'a> From<ObjectIdentifier> for AlgorithmParameters<'a> {
    fn from(oid: ObjectIdentifier) -> AlgorithmParameters<'a> {
        AlgorithmParameters::Oid(oid)
    }
}

impl<'a> TryFrom<Any<'a>> for AlgorithmParameters<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<AlgorithmParameters<'a>> {
        match any.tag() {
            Tag::Null => Null::try_from(any).map(Into::into),
            Tag::ObjectIdentifier => any.oid().map(Into::into),
            _ => Ok(Self::Any(any)),
        }
    }
}

impl<'a> Encodable for AlgorithmParameters<'a> {
    fn encoded_len(&self) -> Result<Length> {
        match self {
            Self::Any(any) => any.encoded_len(),
            Self::Null => Null.encoded_len(),
            Self::Oid(oid) => oid.encoded_len(),
        }
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        match self {
            Self::Any(any) => any.encode(encoder),
            Self::Null => encoder.null(),
            Self::Oid(oid) => encoder.oid(*oid),
        }
    }
}
