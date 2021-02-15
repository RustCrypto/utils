//! Custom derive support for the `der` crate

#![crate_type = "proc-macro"]
#![warn(rust_2018_idioms, trivial_casts, unused_qualifications)]

mod attributes;
mod choice;
mod sequence;
mod types;

use crate::{
    attributes::Asn1Attrs,
    choice::{DeriveDecodableForEnum, DeriveEncodableForEnum},
    sequence::DeriveMessageForStruct,
    types::Asn1Type,
};
use proc_macro2::TokenStream;
use syn::{Generics, Lifetime};
use synstructure::{decl_derive, Structure};

decl_derive!(
    [Decodable, attributes(asn1)] =>

    /// Derive the `Decodable` trait on an enum.
    derive_decodable
);

decl_derive!(
    [Encodable, attributes(asn1)] =>

    /// Derive the `Encodable` trait on an enum.
    derive_encodable
);

decl_derive!(
    [Message, attributes(asn1)] =>

    /// Derive the `Message` trait on a struct.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// `Message` trait for any struct representing a message which is
    /// encoded as an ASN.1 `SEQUENCE`.
    ///
    /// # `#[asn1(type = "...")]` attribute
    ///
    /// Placing this attribute on fields of a struct makes it possible to
    /// decode types which don't directly implement the `Decode` and `Encode`
    /// traits but do impl `TryInto` and `From` for one of the ASN.1 types
    /// listed below:
    ///
    /// - `bit-string`: performs an intermediate conversion to `der::BitString`
    /// - `octet-string`: performs an intermediate conversion to `der::OctetString`
    /// - `printable-string`: performs an intermediate conversion to `der::PrintableString`
    /// - `utf8-string`: performs an intermediate conversion to `der::Utf8String`
    ///
    /// Note: please open a GitHub Issue if you would like to request support
    /// for additional ASN.1 types.
    derive_message
);

/// Custom derive for `der::Decodable`
fn derive_decodable(s: Structure<'_>) -> TokenStream {
    let ast = s.ast();
    let lifetime = parse_lifetime(&ast.generics);

    match &ast.data {
        syn::Data::Enum(data) => DeriveDecodableForEnum::derive(s, data, lifetime),
        other => panic!("can't derive `Decodable` on: {:?}", other),
    }
}

/// Custom derive for `der::Encodable`
fn derive_encodable(s: Structure<'_>) -> TokenStream {
    let ast = s.ast();

    match &ast.data {
        syn::Data::Enum(data) => DeriveEncodableForEnum::derive(s, data),
        other => panic!("can't derive `Encodable` on: {:?}", other),
    }
}

/// Custom derive for `der::Message`
fn derive_message(s: Structure<'_>) -> TokenStream {
    let ast = s.ast();
    let lifetime = parse_lifetime(&ast.generics);

    match &ast.data {
        syn::Data::Struct(data) => DeriveMessageForStruct::derive(s, data, lifetime),
        other => panic!("can't derive `Message` on: {:?}", other),
    }
}

/// Parse the first lifetime of the "self" type of the custom derive
///
/// Returns `None` if there is no first lifetime.
fn parse_lifetime(generics: &Generics) -> Option<&Lifetime> {
    generics
        .lifetimes()
        .next()
        .map(|ref lt_ref| &lt_ref.lifetime)
}
