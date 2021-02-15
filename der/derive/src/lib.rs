//! Custom derive support for the [`der`] crate.
//!
//! This crate contains custom derive macros intended to be used in the
//! following way:
//!
//! - [`Decodable`][`derive@Decodable`] and [`Encodable`][`derive@Encodable`]:
//!   for representing ASN.1 `CHOICE` as a Rust enum
//! - [`Message`][`derive@Message`]: for representing ASN.1 `SEQUENCE` as a Rust struct
//!
//! Note that this crate shouldn't be used directly, but instead accessed
//! by using the `derive` feature of the `der` crate.
//!
//! # `#[asn1(type = "...")]` attribute
//!
//! This attribute can be used to specify the ASN.1 type for a particular
//! enum variant or struct field.
//!
//! It's presently mandatory for all struct fields, even when using one of the
//! ASN.1 types defined by this crate.
//!
//! For structs, placing this attribute on a field makes it possible to
//! decode/encode types which don't directly implement the `Decode`/`Encode`
//! traits but do impl `From` and `TryInto` and `From` for one of the ASN.1 types
//! listed below (use the ASN.1 type keywords as the `type`):
//!
//! - `BIT STRING`: performs an intermediate conversion to [`der::BitString`]
//! - `GeneralizedTime`: performs an intermediate conversion to [`der::GeneralizedTime`]
//! - `OCTET STRING`: performs an intermediate conversion to [`der::OctetString`]
//! - `PrintableString`: performs an intermediate conversion to [`der::PrintableString`]
//! - `UTCTime`: performs an intermediate conversion to [`der::UtcTime`]
//! - `UTF8String`: performs an intermediate conversion to [`der::Utf8String`]
//!
//! Example:
//!
//! ```ignore
//! // NOTE: requires the `derive` feature of `der`
//! use der::{Decodable, Encodable};
//!
//! /// `Time` as defined in RFC 5280
//! #[derive(Decodable, Encodable)]
//! pub enum Time {
//!     #[asn1(type = "UTCTime")]
//!     UtcTime(UtcTime),
//!
//!     #[asn1(type = "GeneralizedTime")]
//!     GeneralTime(GeneralizedTime),
//! }
//! ```
//!
//! Note: please open a GitHub Issue if you would like to request support
//! for additional ASN.1 types.
//!
//! [`der`]: https://docs.rs/der/
//! [`der::BitString`]: https://docs.rs/der/latest/der/struct.BitString.html
//! [`der::GeneralizedTime`]: https://docs.rs/der/latest/der/struct.GeneralizedTime.html
//! [`der::OctetString`]: https://docs.rs/der/latest/der/struct.OctetString.html
//! [`der::PrintableString`]: https://docs.rs/der/latest/der/struct.PrintableString.html
//! [`der::UtcTime`]: https://docs.rs/der/latest/der/struct.UtcTime.html
//! [`der::Utf8String`]: https://docs.rs/der/latest/der/struct.Utf8String.html

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

    /// Derive the [`Decodable`][1] trait on an enum.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// `Decodable` trait for any enum representing a message which is
    /// encoded as an ASN.1 `CHOICE`.
    ///
    /// See [toplevel documentation for the `der_derive` crate][2] for more
    /// information about how to use this macro.
    ///
    /// [1]: https://docs.rs/der/latest/der/trait.Decodable.html
    /// [2]: https://docs.rs/der_derive/
    derive_decodable
);

decl_derive!(
    [Encodable, attributes(asn1)] =>

    /// Derive the [`Encodable`][1] trait on an enum.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// `Encodable` trait for any enum representing a message which is
    /// encoded as an ASN.1 `CHOICE`.
    ///
    /// See [toplevel documentation for the `der_derive` crate][2] for more
    /// information about how to use this macro.
    ///
    /// [1]: https://docs.rs/der/latest/der/trait.Encodable.html
    /// [2]: https://docs.rs/der_derive/
    derive_encodable
);

decl_derive!(
    [Message, attributes(asn1)] =>

    /// Derive the [`Message`][1] trait on a struct.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// `Message` trait for any struct representing a message which is
    /// encoded as an ASN.1 `SEQUENCE`.
    ///
    /// See [toplevel documentation for the `der_derive` crate][2] for more
    /// information about how to use this macro.
    ///
    /// [1]: https://docs.rs/der/latest/der/trait.Message.html
    /// [2]: https://docs.rs/der_derive/
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
