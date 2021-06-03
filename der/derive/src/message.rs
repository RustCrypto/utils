//! Support for deriving the `Message` trait on structs for the purposes of
//! decoding/encoding ASN.1 `SEQUENCE` types as mapped to struct fields.

use crate::{Asn1Attrs, Asn1Type};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataStruct, Field, Ident, Lifetime};
use synstructure::Structure;

/// Derive the `Message` trait for a struct
pub(crate) struct DeriveMessage {
    /// Field decoders
    decode_fields: TokenStream,

    /// Bound fields of a struct to be returned
    decode_result: TokenStream,

    /// Fields of a struct to be serialized
    encode_fields: TokenStream,
}

impl DeriveMessage {
    pub fn derive(s: Structure<'_>, data: &DataStruct, lifetime: Option<&Lifetime>) -> TokenStream {
        let mut state = Self {
            decode_fields: TokenStream::new(),
            decode_result: TokenStream::new(),
            encode_fields: TokenStream::new(),
        };

        for field in &data.fields {
            state.derive_field(field);
        }

        state.finish(&s, lifetime)
    }

    /// Derive handling for a particular `#[field(...)]`
    fn derive_field(&mut self, field: &Field) {
        let name = field
            .ident
            .as_ref()
            .cloned()
            .expect("no name on struct field i.e. tuple structs unsupported");

        let asn1_type = Asn1Attrs::new(&field.attrs).asn1_type;
        self.derive_field_decoder(&name, asn1_type);
        self.derive_field_encoder(&name, asn1_type);
    }

    /// Derive code for decoding a field of a message
    fn derive_field_decoder(&mut self, name: &Ident, asn1_type: Option<Asn1Type>) {
        let field_decoder = match asn1_type {
            Some(Asn1Type::BitString) => quote! {
                let #name = decoder.bit_string()?.try_into()?;
            },
            Some(Asn1Type::GeneralizedTime) => quote! {
                let #name = decoder.generalized_time()?.try_into()?;
            },
            Some(Asn1Type::OctetString) => quote! {
                let #name = decoder.octet_string()?.try_into()?;
            },
            Some(Asn1Type::PrintableString) => quote! {
                let #name = decoder.printable_string()?.try_into()?;
            },
            Some(Asn1Type::UtcTime) => quote! {
                let #name = decoder.utc_time()?.try_into()?;
            },
            Some(Asn1Type::Utf8String) => quote! {
                let #name = decoder.utf8_string()?.try_into()?;
            },
            None => quote! { let #name = decoder.decode()?; },
        };
        field_decoder.to_tokens(&mut self.decode_fields);

        let field_result = quote!(#name,);
        field_result.to_tokens(&mut self.decode_result);
    }

    /// Derive code for encoding a field of a message
    fn derive_field_encoder(&mut self, name: &Ident, asn1_type: Option<Asn1Type>) {
        let binding = quote!(&self.#name);
        asn1_type
            .map(|ty| {
                let encoder = ty.encoder(binding.clone());
                quote!(&#encoder?,)
            })
            .unwrap_or_else(|| quote!(#binding,))
            .to_tokens(&mut self.encode_fields);
    }

    /// Finish deriving a struct
    fn finish(self, s: &Structure<'_>, lifetime: Option<&Lifetime>) -> TokenStream {
        let lifetime = match lifetime {
            Some(lifetime) => quote!(#lifetime),
            None => quote!('_),
        };

        let decode_fields = self.decode_fields;
        let decode_result = self.decode_result;
        let encode_fields = self.encode_fields;

        s.gen_impl(quote! {
            gen impl core::convert::TryFrom<::der::asn1::Any<#lifetime>> for @Self {
                type Error = ::der::Error;

                fn try_from(any: ::der::asn1::Any<#lifetime>) -> ::der::Result<Self> {
                    #[allow(unused_imports)]
                    use core::convert::TryInto;

                    any.sequence(|decoder| {
                        #decode_fields
                        Ok(Self { #decode_result })
                    })
                }
            }

            gen impl ::der::Message<#lifetime> for @Self {
                fn fields<F, T>(&self, f: F) -> ::der::Result<T>
                where
                    F: FnOnce(&[&dyn der::Encodable]) -> ::der::Result<T>,
                {
                    #[allow(unused_imports)]
                    use core::convert::TryFrom;

                    f(&[#encode_fields])
                }
            }
        })
    }
}
