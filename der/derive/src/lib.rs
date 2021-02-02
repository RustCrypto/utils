//! Custom derive support for the `der` crate

#![crate_type = "proc-macro"]
#![warn(rust_2018_idioms, trivial_casts, unused_qualifications)]

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    DataStruct, Field, Generics, Ident, Lifetime, Lit, Meta, MetaList, MetaNameValue, NestedMeta,
};
use synstructure::{decl_derive, Structure};

decl_derive!(
    [Message, attributes(asn1)] =>

    /// Derive the `Message` trait.
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
    derive_der_message
);

/// Custom derive for `der::Message`
fn derive_der_message(s: Structure<'_>) -> TokenStream {
    let ast = s.ast();

    // TODO(tarcieri): enum support
    match &ast.data {
        syn::Data::Struct(data) => DeriveStruct::derive(s, data, &ast.generics),
        other => panic!("can't derive `Message` on: {:?}", other),
    }
}

/// Derive `Message` on a struct
// TODO(tarcieri): make sure tags are in the right order and digest is the last field
struct DeriveStruct {
    /// Field decoders
    decode_fields: TokenStream,

    /// Bound fields of a struct to be returned
    decode_result: TokenStream,

    /// Fields of a struct to be serialized
    encode_fields: TokenStream,
}

impl DeriveStruct {
    pub fn derive(s: Structure<'_>, data: &DataStruct, generics: &Generics) -> TokenStream {
        let mut state = Self {
            decode_fields: TokenStream::new(),
            decode_result: TokenStream::new(),
            encode_fields: TokenStream::new(),
        };

        for field in &data.fields {
            state.derive_field(field);
        }

        state.finish(&s, generics)
    }

    /// Derive handling for a particular `#[field(...)]`
    fn derive_field(&mut self, field: &Field) {
        let attrs = FieldAttrs::new(field);
        self.derive_field_decoder(&attrs);
        self.derive_field_encoder(&attrs);
    }

    /// Derive code for decoding a field of a message
    fn derive_field_decoder(&mut self, field: &FieldAttrs) {
        let field_name = &field.name;
        let field_decoder = match field.asn1_type {
            Some(Asn1Type::BitString) => {
                quote! { let #field_name = decoder.bit_string()?.try_into()?; }
            }
            Some(Asn1Type::OctetString) => {
                quote! { let #field_name = decoder.octet_string()?.try_into()?; }
            }
            Some(Asn1Type::PrintableString) => {
                quote! { let #field_name = decoder.printable_string()?.try_into()?; }
            }
            Some(Asn1Type::Utf8String) => {
                quote! { let #field_name = decoder.utf8_string()?.try_into()?; }
            }
            None => quote! { let #field_name = decoder.decode()?; },
        };
        field_decoder.to_tokens(&mut self.decode_fields);

        let field_result = quote!(#field_name,);
        field_result.to_tokens(&mut self.decode_result);
    }

    /// Derive code for encoding a field of a message
    fn derive_field_encoder(&mut self, field: &FieldAttrs) {
        let field_name = &field.name;
        let field_encoder = match field.asn1_type {
            Some(Asn1Type::BitString) => {
                quote!(&der::BitString::new(&self.#field_name)?,)
            }
            Some(Asn1Type::OctetString) => {
                quote!(&der::OctetString::new(&self.#field_name)?,)
            }
            Some(Asn1Type::PrintableString) => {
                quote!(&der::PrintableString::new(&self.#field_name)?,)
            }
            Some(Asn1Type::Utf8String) => {
                quote!(&der::Utf8String::new(&self.#field_name)?,)
            }
            None => quote!(&self.#field_name,),
        };
        field_encoder.to_tokens(&mut self.encode_fields);
    }

    /// Finish deriving a struct
    fn finish(self, s: &Structure<'_>, generics: &Generics) -> TokenStream {
        let lifetime = match parse_lifetime(generics) {
            Some(lifetime) => quote!(#lifetime),
            None => quote!('_),
        };

        let decode_fields = self.decode_fields;
        let decode_result = self.decode_result;
        let encode_fields = self.encode_fields;

        s.gen_impl(quote! {
            gen impl core::convert::TryFrom<der::Any<#lifetime>> for @Self {
                type Error = der::Error;

                fn try_from(any: der::Any<#lifetime>) -> der::Result<Self> {
                    #[allow(unused_imports)]
                    use core::convert::TryInto;

                    any.sequence(|decoder| {
                        #decode_fields
                        Ok(Self { #decode_result })
                    })
                }
            }

            gen impl der::Message<#lifetime> for @Self {
                fn fields<F, T>(&self, f: F) -> der::Result<T>
                where
                    F: FnOnce(&[&dyn der::Encodable]) -> der::Result<T>,
                {
                    f(&[#encode_fields])
                }
            }
        })
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

/// Attributes of a field
#[derive(Debug)]
struct FieldAttrs {
    /// Name of the field
    pub name: Ident,

    /// Value of the `#[asn1(type = "...")]` attribute if provided
    pub asn1_type: Option<Asn1Type>,
}

impl FieldAttrs {
    /// Parse the attributes of a field
    fn new(field: &Field) -> Self {
        let name = field
            .ident
            .as_ref()
            .cloned()
            .expect("no name on struct field i.e. tuple structs unsupported");

        let mut asn1_type = None;

        for attr in &field.attrs {
            if !attr.path.is_ident("asn1") {
                continue;
            }

            match attr.parse_meta().expect("error parsing `asn1` attribute") {
                Meta::List(MetaList { nested, .. }) if nested.len() == 1 => {
                    match nested.first() {
                        Some(NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(lit_str),
                            ..
                        }))) => {
                            // Parse the `type = "..."` attribute
                            if !path.is_ident("type") {
                                panic!("unknown `asn1` attribute for field `{}`: {:?}", name, path);
                            }

                            if asn1_type.is_some() {
                                panic!("duplicate ASN.1 `type` attribute for field: {}", name);
                            }

                            asn1_type = Some(Asn1Type::new(&lit_str.value()));
                        }
                        other => panic!(
                            "malformed `asn1` attribute for field `{}`: {:?}",
                            name, other
                        ),
                    }
                }
                other => panic!(
                    "malformed `asn1` attribute for field `{}`: {:?}",
                    name, other
                ),
            }
        }

        Self { name, asn1_type }
    }
}

/// ASN.1 built-in types supported by the `#[asn1(type = "...")]` attribute
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
enum Asn1Type {
    /// ASN.1 `BIT STRING`
    BitString,

    /// ASN.1 `OCTET STRING`
    OctetString,

    /// ASN.1 `PrintableString`
    PrintableString,

    /// ASN.1 `UTF8String`
    Utf8String,
}

impl Asn1Type {
    /// Parse ASN.1 type
    pub fn new(s: &str) -> Self {
        match s {
            "bit-string" => Self::BitString,
            "octet-string" => Self::OctetString,
            "printable-string" => Self::PrintableString,
            "utf8-string" => Self::Utf8String,
            _ => panic!("unrecognized ASN.1 type: {}", s),
        }
    }
}
