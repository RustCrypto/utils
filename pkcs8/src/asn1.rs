//! ASN.1 decoder and encoder

pub(crate) mod decoder;
pub(crate) mod encoder;

/// ASN.1 tags
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Tag {
    /// ASN.1 `INTEGER` tag
    Integer = 0x02,

    /// ASN.1 `BIT STRING` tag
    BitString = 0x03,

    /// ASN.1 `OCTET STRING` tag
    OctetString = 0x04,

    /// ASN.1 `NULL` tag
    Null = 0x05,

    /// ASN.1 `OBJECT IDENTIFIER` tag
    ObjectIdentifier = 0x06,

    /// ASN.1 `SEQUENCE` tag
    Sequence = 0x30,
}
