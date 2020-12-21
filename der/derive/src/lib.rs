//! Custom derive support for the `der` crate

#![crate_type = "proc-macro"]
#![warn(rust_2018_idioms, trivial_casts, unused_qualifications)]

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataStruct, Field, Generics, Ident, Lifetime};
use synstructure::{decl_derive, Structure};

decl_derive!(
    [Message] =>

    /// Derive the [`Message`] trait.
    ///
    /// This custom derive macro can be used to automatically impl the
    /// [`Message`] trait for any struct representing a message which is
    /// encoded as an ASN.1 `SEQUENCE`.
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
        // Rust models structs as enums with a single variant
        assert_eq!(s.variants().len(), 1, "expected one variant");

        let mut state = Self {
            decode_fields: TokenStream::new(),
            decode_result: TokenStream::new(),
            encode_fields: TokenStream::new(),
        };

        let variant = &s.variants()[0];
        let bindings = &variant.bindings();

        assert_eq!(
            bindings.len(),
            data.fields.len(),
            "unexpected number of bindings ({} vs {})",
            bindings.len(),
            data.fields.len()
        );

        for (binding_info, field) in bindings.iter().zip(&data.fields) {
            state.derive_field(field, &binding_info.binding);
        }

        state.finish(&s, generics)
    }

    /// Derive handling for a particular `#[field(...)]`
    fn derive_field(&mut self, field: &Field, _binding: &Ident) {
        let name = parse_field_name(field);
        self.derive_field_decoder(name);
        self.derive_field_encoder(name);
    }

    /// Derive code for decoding a field of a message
    fn derive_field_decoder(&mut self, name: &Ident) {
        let field_decoder = quote! { let #name = decoder.decode()?; };
        field_decoder.to_tokens(&mut self.decode_fields);

        let field_result = quote!(#name,);
        field_result.to_tokens(&mut self.decode_result);
    }

    /// Derive code for encoding a field of a message
    fn derive_field_encoder(&mut self, name: &Ident) {
        let field_encoder = quote!(&self.#name,);
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

/// Parse the name of a field
fn parse_field_name(field: &Field) -> &Ident {
    field
        .ident
        .as_ref()
        .unwrap_or_else(|| panic!("no name on struct field (e.g. tuple structs unsupported)"))
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
