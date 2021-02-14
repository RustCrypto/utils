//! Attribute-related types used by the proc macro

use crate::Asn1Type;
use syn::{Attribute, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

#[derive(Debug)]
pub(crate) struct Asn1Attrs {
    /// Value of the `#[asn1(type = "...")]` attribute if provided
    pub asn1_type: Option<Asn1Type>,
}

impl Asn1Attrs {
    /// Parse attributes from a field or enum variant
    pub fn new(attrs: &[Attribute]) -> Self {
        let mut asn1_type = None;

        for attr in attrs {
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
                                panic!("unknown `asn1` attribute: {:?}", path);
                            }

                            if let Some(ty) = asn1_type {
                                panic!("duplicate ASN.1 `type` attribute: {:?}", ty);
                            }

                            asn1_type = Some(Asn1Type::new(&lit_str.value()));
                        }
                        other => panic!("malformed `asn1` attribute: {:?}", other),
                    }
                }
                other => panic!("malformed `asn1` attribute: {:?}", other),
            }
        }

        Self { asn1_type }
    }
}
