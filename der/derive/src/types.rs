//! ASN.1 types supported by the proc macro

use core::fmt;
use proc_macro2::TokenStream;
use quote::quote;

/// ASN.1 built-in types supported by the `#[asn1(type = "...")]` attribute
// TODO(tarcieri): support all ASN.1 types specified in `der::Tag`
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub(crate) enum Asn1Type {
    /// ASN.1 `BIT STRING`
    BitString,

    /// ASN.1 `GeneralizedTime`
    GeneralizedTime,

    /// ASN.1 `OCTET STRING`
    OctetString,

    /// ASN.1 `PrintableString`
    PrintableString,

    /// ASN.1 `UTCTime`
    UtcTime,

    /// ASN.1 `UTF8String`
    Utf8String,
}

impl Asn1Type {
    /// Parse ASN.1 type
    pub fn new(s: &str) -> Self {
        match s {
            "BIT STRING" => Self::BitString,
            "GeneralizedTime" => Self::GeneralizedTime,
            "OCTET STRING" => Self::OctetString,
            "PrintableString" => Self::PrintableString,
            "UTCTime" => Self::UtcTime,
            "UTF8String" => Self::Utf8String,
            _ => panic!("unrecognized ASN.1 type: {}", s),
        }
    }

    /// Get the `::der::Tag` for this ASN.1 type
    pub fn tag(&self) -> TokenStream {
        match self {
            Asn1Type::BitString => quote!(::der::Tag::BitString),
            Asn1Type::GeneralizedTime => quote!(::der::Tag::GeneralizedTime),
            Asn1Type::OctetString => quote!(::der::Tag::OctetString),
            Asn1Type::PrintableString => quote!(::der::Tag::PrintableString),
            Asn1Type::UtcTime => quote!(::der::Tag::UtcTime),
            Asn1Type::Utf8String => quote!(::der::Tag::Utf8String),
        }
    }

    /// Get a `der::Encoder` object for a particular ASN.1 type
    pub fn encoder(&self, binding: TokenStream) -> TokenStream {
        match self {
            Asn1Type::BitString => quote!(::der::asn1::BitString::new(#binding)),
            Asn1Type::GeneralizedTime => quote!(::der::asn1::GeneralizedTime::try_from(#binding)),
            Asn1Type::OctetString => quote!(::der::asn1::OctetString::new(#binding)),
            Asn1Type::PrintableString => quote!(::der::asn1::PrintableString::new(#binding)),
            Asn1Type::UtcTime => quote!(::der::asn1::UtcTime::try_from(#binding)),
            Asn1Type::Utf8String => quote!(&::der::Utf8String::new(#binding)),
        }
    }
}

impl fmt::Display for Asn1Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Asn1Type::BitString => "BIT STRING",
            Asn1Type::GeneralizedTime => "GeneralizedTime",
            Asn1Type::OctetString => "OCTET STRING",
            Asn1Type::PrintableString => "PrintableString",
            Asn1Type::UtcTime => "UTCTime",
            Asn1Type::Utf8String => "UTF8String",
        })
    }
}
