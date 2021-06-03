//! X.509 Attributes

use der::{
    asn1::{Any, ObjectIdentifier},
    Message,
};

/// Attribute type/value pairs as defined in [RFC 5280 Section 4.1.2.4].
///
/// ```text
/// AttributeTypeAndValue ::= SEQUENCE {
///   type     AttributeType,
///   value    AttributeValue }
///
/// AttributeType ::= OBJECT IDENTIFIER
///
/// AttributeValue ::= ANY -- DEFINED BY AttributeType
/// ```
///
/// [RFC 5280 Section 4.1.2.4]: https://tools.ietf.org/html/rfc5280#section-4.1.2.4
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Message)]
pub struct AttributeTypeAndValue<'a> {
    /// OID describing the type of the attribute
    pub oid: ObjectIdentifier,

    /// Value of the attribute
    pub value: Any<'a>,
}
