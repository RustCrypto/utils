//! PKCS#8 attributes.

use core::convert::TryFrom;
use der::{asn1::Any, Encodable, Encoder, Length};

/// Attributes as defined in [RFC 5958 Section 2].
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
// TODO(tarcieri): support parsing attributes as a `der::SetOf`?
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Attributes<'a>(Any<'a>);

impl<'a> From<Attributes<'a>> for Any<'a> {
    fn from(attrs: Attributes<'a>) -> Any<'a> {
        attrs.0
    }
}

impl<'a> TryFrom<Any<'a>> for Attributes<'a> {
    type Error = der::Error;

    fn try_from(any: Any<'a>) -> der::Result<Attributes<'a>> {
        Ok(Attributes(any))
    }
}

impl<'a> Encodable for Attributes<'a> {
    /// Compute the length of this value in bytes when encoded as ASN.1 DER.
    fn encoded_len(&self) -> der::Result<Length> {
        self.0.encoded_len()
    }

    /// Encode this value as ASN.1 DER using the provided [`Encoder`].
    fn encode(&self, encoder: &mut Encoder<'_>) -> der::Result<()> {
        self.0.encode(encoder)
    }
}
