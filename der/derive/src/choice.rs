//! Support for deriving the `Decodable` and `Encodable` traits on enums for
//! the purposes of decoding/encoding ASN.1 `CHOICE` types as mapped to
//! enum variants.

use crate::{Asn1Attrs, Asn1Type};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataEnum, Lifetime};
use synstructure::{Structure, VariantInfo};

/// Derive `Decodable` for an enum.
pub(crate) struct DeriveDecodableForEnum {
    /// Enum match arms for the impl body for `TryFrom<der::Any<'_>>`
    decode_body: TokenStream,
}

impl DeriveDecodableForEnum {
    /// Derive `Decodable` on an enum.
    pub fn derive(s: Structure<'_>, data: &DataEnum, lifetime: Option<&Lifetime>) -> TokenStream {
        assert_eq!(
            s.variants().len(),
            data.variants.len(),
            "enum variant count mismatch"
        );

        let mut state = Self {
            decode_body: TokenStream::new(),
        };

        for variant in &data.variants {
            let asn1_type = Asn1Attrs::new(&variant.attrs).asn1_type.unwrap_or_else(|| {
                panic!(
                    "no #[asn1(type=...)] specified for enum variant: {}",
                    variant.ident
                )
            });

            state.derive_variant_decoder(asn1_type);
        }

        state.finish(s, lifetime)
    }

    /// Derive a match arm of the impl body for `TryFrom<der::Any<'_>>`.
    fn derive_variant_decoder(&mut self, asn1_type: Asn1Type) {
        let tag = asn1_type.tag();

        let decoder = match asn1_type {
            Asn1Type::BitString => quote!(any.bit_string()),
            Asn1Type::GeneralizedTime => quote!(any.generalized_time()),
            Asn1Type::OctetString => quote!(any.octet_string()),
            Asn1Type::PrintableString => quote!(any.printable_string()),
            Asn1Type::UtcTime => quote!(any.utc_time()),
            Asn1Type::Utf8String => quote!(any.utf8_string()),
        };

        {
            quote! {
                #tag => {
                    #decoder.ok().and_then(|val| val.try_into().ok()).ok_or_else(|| {
                        ::der::ErrorKind::Value { tag: #tag }.into()
                    })
                }
            }
        }
        .to_tokens(&mut self.decode_body);
    }

    /// Finish deriving an enum
    fn finish(self, s: Structure<'_>, lifetime: Option<&Lifetime>) -> TokenStream {
        let lifetime = match lifetime {
            Some(lifetime) => quote!(#lifetime),
            None => quote!('_),
        };

        let decode_body = self.decode_body;

        s.gen_impl(quote! {
            gen impl core::convert::TryFrom<der::Any<#lifetime>> for @Self {
                type Error = der::Error;

                fn try_from(any: der::Any<#lifetime>) -> der::Result<Self> {
                    #[allow(unused_imports)]
                    use core::convert::TryInto;

                    match any.tag() {
                        #decode_body
                        actual => Err(der::ErrorKind::UnexpectedTag {
                            expected: None,
                            actual
                        }
                        .into()),
                    }
                }
            }
        })
    }
}

/// Derive `Encodable` for an enum.
pub(crate) struct DeriveEncodableForEnum {
    /// Enum match arms for the impl body for `der::Encodable::encode`
    encode_body: TokenStream,

    /// Enum match arms for the impl body for `der::Encodable::encoded_len`
    encoded_len_body: TokenStream,
}

impl DeriveEncodableForEnum {
    /// Derive `Encodable` on an enum.
    pub fn derive(s: Structure<'_>, data: &DataEnum) -> TokenStream {
        assert_eq!(
            s.variants().len(),
            data.variants.len(),
            "enum variant count mismatch"
        );

        let mut state = Self {
            encode_body: TokenStream::new(),
            encoded_len_body: TokenStream::new(),
        };

        for (variant_info, variant) in s.variants().iter().zip(&data.variants) {
            let asn1_type = Asn1Attrs::new(&variant.attrs).asn1_type.unwrap_or_else(|| {
                panic!(
                    "no #[asn1(type=...)] specified for enum variant: {}",
                    variant.ident
                )
            });

            match variant_info.bindings().len() {
                // TODO(tarcieri): handle 0 bindings for ASN.1 NULL
                1 => {
                    state.derive_variant_encoder(&variant_info, asn1_type);
                    state.derive_variant_encoded_len(&variant_info);
                }
                other => panic!(
                    "unsupported number of ASN.1 variant bindings for {}: {}",
                    asn1_type, other
                ),
            }
        }

        state.finish(s)
    }

    /// Derive a match arm for the impl body for `der::Encodable::encode`.
    fn derive_variant_encoder(&mut self, variant: &VariantInfo<'_>, asn1_type: Asn1Type) {
        assert_eq!(
            variant.bindings().len(),
            1,
            "unexpected number of variant bindings"
        );

        variant
            .each(|bi| {
                let binding = &bi.binding;
                let encoder_obj = asn1_type.encoder(quote!(#binding));
                quote!(#encoder_obj?.encode(encoder))
            })
            .to_tokens(&mut self.encode_body);
    }

    /// Derive a match arm for the impl body for `der::Encodable::encode`.
    fn derive_variant_encoded_len(&mut self, variant: &VariantInfo<'_>) {
        assert_eq!(
            variant.bindings().len(),
            1,
            "unexpected number of variant bindings"
        );

        variant
            .each(|bi| {
                let binding = &bi.binding;
                quote!(#binding.encoded_len())
            })
            .to_tokens(&mut self.encoded_len_body);
    }

    /// Finish deriving an enum
    fn finish(self, s: Structure<'_>) -> TokenStream {
        let encode_body = self.encode_body;
        let encoded_len_body = self.encoded_len_body;

        s.gen_impl(quote! {
            gen impl ::der::Encodable for @Self {
                fn encode(&self, encoder: &mut ::der::Encoder<'_>) -> ::der::Result<()> {
                    #[allow(unused_imports)]
                    use core::convert::TryFrom;

                    match self {
                        #encode_body
                    }
                }

                fn encoded_len(&self) -> ::der::Result<::der::Length> {
                    match self {
                        #encoded_len_body
                    }
                }
            }
        })
    }
}
