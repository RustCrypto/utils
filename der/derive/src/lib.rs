//! Custom derive support for the [`der`] crate.
//!
//! This crate contains custom derive macros intended to be used in the
//! following way:
//!
//! - [`Choice`][`derive@Choice`]: map ASN.1 `CHOICE` to a Rust enum.
//! - [`Message`][`derive@Message`]: map ASN.1 `SEQUENCE` to a Rust struct.
//!
//! Note that this crate shouldn't be used directly, but instead accessed
//! by using the `derive` feature of the `der` crate.
//!
//! # Why not `serde`?
//!
//! The `der` crate is designed to be easily usable in embedded environments,
//! including ones where code size comes at a premium.
//!
//! This crate (i.e. `der_derive`) is able to generate code which is
//! significantly smaller than `serde_derive`. This is because the `der`
//! crate has been designed with high-level abstractions which reduce
//! code size, including trait object-based encoders which allow encoding
//! logic which is duplicated in `serde` serializers to be implemented in
//! a single place in the `der` crate.
//!
//! This is a deliberate tradeoff in terms of performance, flexibility, and
//! code size. At least for now, the `der` crate is optimizing for leveraging
//! as many abstractions as it can to minimize code size.
//!
//! # `#[asn1(type = "...")]` attribute
//!
//! This attribute can be used to specify the ASN.1 type for a particular
//! enum variant or struct field.
//!
//! It's presently mandatory for all enum variants, even when using one of the
//! ASN.1 types defined by this crate.
//!
//! For structs, placing this attribute on a field makes it possible to
//! decode/encode types which don't directly implement the `Decode`/`Encode`
//! traits but do impl `From` and `TryInto` and `From` for one of the ASN.1 types
//! listed below (use the ASN.1 type keywords as the `type`):
//!
//! - `BIT STRING`: performs an intermediate conversion to [`der::asn1::BitString`]
//! - `GeneralizedTime`: performs an intermediate conversion to [`der::asn1::GeneralizedTime`]
//! - `OCTET STRING`: performs an intermediate conversion to [`der::asn1::OctetString`]
//! - `PrintableString`: performs an intermediate conversion to [`der::asn1::PrintableString`]
//! - `UTCTime`: performs an intermediate conversion to [`der::asn1::UtcTime`]
//! - `UTF8String`: performs an intermediate conversion to [`der::asn1::Utf8String`]
//!
//! Note: please open a GitHub Issue if you would like to request support
//! for additional ASN.1 types.
//!
//! [`der`]: https://docs.rs/der/
//! [`der::asn1::BitString`]: https://docs.rs/der/latest/der/asn1/struct.BitString.html
//! [`der::asn1::GeneralizedTime`]: https://docs.rs/der/latest/der/asn1/struct.GeneralizedTime.html
//! [`der::asn1::OctetString`]: https://docs.rs/der/latest/der/asn1/struct.OctetString.html
//! [`der::asn1::PrintableString`]: https://docs.rs/der/latest/der/asn1/struct.PrintableString.html
//! [`der::asn1::UtcTime`]: https://docs.rs/der/latest/der/asn1/struct.UtcTime.html
//! [`der::asn1::Utf8String`]: https://docs.rs/der/latest/der/asn1/struct.Utf8String.html

#![crate_type = "proc-macro"]
#![warn(rust_2018_idioms, trivial_casts, unused_qualifications)]

mod attributes;
mod choice;
mod message;
mod types;

use crate::{attributes::Asn1Attrs, choice::DeriveChoice, message::DeriveMessage, types::Asn1Type};
use proc_macro2::TokenStream;
use syn::{Generics, Lifetime};
use synstructure::{decl_derive, Structure};

decl_derive!(
    [Choice, attributes(asn1)] =>

    /// Derive the [`Choice`][1] trait on an enum.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// [`Decodable`][2] and [`Encodable`][3] traits along with the
    /// [`Choice`][1] supertrait for any enum representing an ASN.1 `CHOICE`.
    ///
    /// The enum must consist entirely of 1-tuple variants wrapping inner
    /// types which must also impl the [`Decodable`][2] and [`Encodable`][3]
    /// traits. It will will also generate [`From`] impls for each of the
    /// inner types of the variants into the enum that wraps them.
    ///
    /// # Usage
    ///
    /// ```ignore
    /// // NOTE: requires the `derive` feature of `der`
    /// use der::Choice;
    ///
    /// /// `Time` as defined in RFC 5280
    /// #[derive(Choice)]
    /// pub enum Time {
    ///     #[asn1(type = "UTCTime")]
    ///     UtcTime(UtcTime),
    ///
    ///     #[asn1(type = "GeneralizedTime")]
    ///     GeneralTime(GeneralizedTime),
    /// }
    /// ```
    ///
    /// # `#[asn1(type = "...")]` attribute
    ///
    /// See [toplevel documentation for the `der_derive` crate][4] for more
    /// information about the `#[asn1]` attribute.
    ///
    /// [1]: https://docs.rs/der/latest/der/trait.Choice.html
    /// [2]: https://docs.rs/der/latest/der/trait.Decodable.html
    /// [3]: https://docs.rs/der/latest/der/trait.Encodable.html
    /// [4]: https://docs.rs/der_derive/
    derive_choice
);

decl_derive!(
    [Message, attributes(asn1)] =>

    /// Derive the [`Message`][1] trait on a struct.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// `Message` trait for any struct representing a message which is
    /// encoded as an ASN.1 `SEQUENCE`.
    ///
    /// # Usage
    ///
    /// ```ignore
    /// use der::{
    ///     asn1::{Any, ObjectIdentifier},
    ///     Message
    /// };
    ///
    /// /// X.509 `AlgorithmIdentifier`
    /// #[derive(Message)]
    /// pub struct AlgorithmIdentifier<'a> {
    ///     /// This field contains an ASN.1 `OBJECT IDENTIFIER`, a.k.a. OID.
    ///     pub algorithm: ObjectIdentifier,
    ///
    ///     /// This field is `OPTIONAL` and contains the ASN.1 `ANY` type, which
    ///     /// in this example allows arbitrary algorithm-defined parameters.
    ///     pub parameters: Option<Any<'a>>
    /// }
    /// ```
    ///
    /// # `#[asn1(type = "...")]` attribute
    ///
    /// See [toplevel documentation for the `der_derive` crate][2] for more
    /// information about the `#[asn1]` attribute.
    ///
    /// [1]: https://docs.rs/der/latest/der/trait.Message.html
    /// [2]: https://docs.rs/der_derive/
    derive_message
);

/// Custom derive for `der::Choice`
fn derive_choice(s: Structure<'_>) -> TokenStream {
    let ast = s.ast();
    let lifetime = parse_lifetime(&ast.generics);

    match &ast.data {
        syn::Data::Enum(data) => DeriveChoice::derive(s, data, lifetime),
        other => panic!("can't derive `Choice` on: {:?}", other),
    }
}

/// Custom derive for `der::Message`
fn derive_message(s: Structure<'_>) -> TokenStream {
    let ast = s.ast();
    let lifetime = parse_lifetime(&ast.generics);

    match &ast.data {
        syn::Data::Struct(data) => DeriveMessage::derive(s, data, lifetime),
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
