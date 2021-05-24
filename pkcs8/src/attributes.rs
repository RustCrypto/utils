//! `OneAsymmetricKey` attributes.

use core::convert::TryFrom;

use der::{Any, Encodable, Encoder, Error, Header, Length, Result, Tag, Tagged};

/// Attributes as defined in [RFC 5958 Section 2]:
///
/// > attributes is OPTIONAL.  It contains information corresponding to
/// > the public key (e.g., certificates).  The attributes field uses the
/// > class `ATTRIBUTE` which is restricted by the
/// > `OneAsymmetricKeyAttributes` information object set.
/// > `OneAsymmetricKeyAttributes` is an open ended set in this document.
/// > Others documents can constrain these values.  Attributes from
/// > RFC2985 MAY be supported.
///
/// Attributes have the following ASN.1 schema:
///
/// ```text
/// Attributes ::= SET OF Attribute { { OneAsymmetricKeyAttributes } }
/// ```
///
/// [RFC 5958 Section 2]: https://datatracker.ietf.org/doc/html/rfc5958
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Attributes<'a> {
    /// Inner ASN.1 value (i.e. `SET OF Attribute`).
    // TODO(tarcieri): decode this as a `SetOf`?
    inner: Any<'a>,
}

impl<'a> Attributes<'a> {
    /// Create an [`Attributes`] wrapper for the given [`Any`] value.
    ///
    /// Note that no validation of this value is performed. It is assumed to be
    /// a well-structured set of `OneAsymmetricKeyAttributes`.
    pub fn from_raw_attrs(attrs: Any<'a>) -> Self {
        Self { inner: attrs }
    }

    /// Borrow the inner attributes value as an [`Any`].
    // TODO(tarcieri): decode this as a `SetOf` and allow iteration?
    pub fn as_any(&self) -> &Any<'a> {
        &self.inner
    }
}

impl<'a> TryFrom<Any<'a>> for Attributes<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Self::TAG)?;
        let inner = Any::try_from(any.as_bytes())?;
        Ok(Self { inner })
    }
}

impl<'a> Encodable for Attributes<'a> {
    fn encoded_len(&self) -> Result<Length> {
        self.inner.encoded_len()?.for_tlv()
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        Header::new(Self::TAG, self.inner.encoded_len()?)?.encode(encoder)?;
        self.inner.encode(encoder)
    }
}

impl<'a> Tagged for Attributes<'a> {
    const TAG: Tag = Tag::ContextSpecific0;
}
