//! Postprocessor for fiat-crypto generated field implementations which rewrites
//! them as `const fn`.
//!
//! Usage: fiat-constify /path/to/field_impl.rs

#![allow(clippy::single_match, clippy::new_without_default)]

mod type_registry;

use proc_macro2::{Punct, Spacing, Span};
use quote::TokenStreamExt;
use std::{env, fs, ops::Deref};
use syn::{FnArg, Ident, Item, ItemFn, Meta, Pat, Stmt, TypeReference, parse_quote, token::Const};
use type_registry::TypeRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        panic!("Usage: fiat-constify /path/to/field_impl.rs")
    }

    let code = fs::read_to_string(&args[1])?;
    let mut ast = syn::parse_file(&code)?;
    ast.attrs.push(parse_quote! {
        #![allow(
            clippy::identity_op,
            clippy::too_many_arguments,
            clippy::unnecessary_cast,
            dead_code
        )]
    });

    let mut type_registry = TypeRegistry::new();

    // Iterate over functions, transforming them into `const fn`
    for item in &mut ast.items {
        match item {
            Item::Fn(func) => rewrite_fn_as_const(func, &type_registry),
            Item::Type(ty) => type_registry.add_type_alias(ty),
            Item::Struct(ty) => {
                if let Some(derive) = ty
                    .attrs
                    .iter_mut()
                    .find(|x| x.meta.path().is_ident("derive"))
                {
                    ["Debug", "PartialEq", "Eq", "PartialOrd", "Ord"]
                        .iter()
                        .for_each(|x| {
                            if let Meta::List(derive_list) = &mut derive.meta {
                                derive_list.tokens.append(Punct::new(',', Spacing::Alone));
                                derive_list
                                    .tokens
                                    .append(proc_macro2::Ident::new(x, Span::call_site()));
                            }
                        });
                }

                type_registry.add_newtype(ty)
            }
            _ => (),
        }
    }

    println!(
        "//! fiat-crypto output postprocessed by fiat-constify: <https://github.com/rustcrypto/utils>"
    );
    println!("{}", prettyplease::unparse(&ast));
    Ok(())
}

/// Get an `Ident` from a `Pat::Ident`.
fn get_ident_from_pat(pat: &Pat) -> Ident {
    match pat {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        other => panic!("unexpected `Pat`: {other:?} (expecting `Pat::Ident`)"),
    }
}

/// Rewrite a fiat-crypto generated `fn` as a `const fn`, making the necessary
/// transformations to the code in order for it to work in that context.
fn rewrite_fn_as_const(func: &mut ItemFn, type_registry: &TypeRegistry) {
    // Mark function as being `const fn`.
    func.sig.constness = Some(Const::default());

    // Transform mutable arguments into return values.
    let mut stmts = Vec::<Stmt>::new();

    for arg in &func.sig.inputs {
        // Transform mutable function arguments into return values
        if let FnArg::Typed(t) = arg {
            match &*t.ty {
                syn::Type::Reference(TypeReference {
                    mutability: Some(_), // look for mutable references
                    elem,
                    ..
                }) => {
                    if matches!(elem.deref(), syn::Type::Path(_)) {
                        // Generation of reborrows, LLVM should optimize this out, and it definitely
                        // will if `#[repr(transparent)]` is used.
                        let ty = type_registry::type_to_ident(elem).unwrap();
                        let ident = get_ident_from_pat(&t.pat);
                        if type_registry.is_newtype(ty) {
                            stmts.push(parse_quote! {
                                let #ident = &mut #ident.0;
                            });
                        }
                    }
                }
                syn::Type::Reference(TypeReference {
                    mutability: None,
                    elem,
                    ..
                }) if matches!(elem.deref(), syn::Type::Path(_)) => {
                    // Generation of reborrows, LLVM should optimize this out, and it definitely
                    // will if `#[repr(transparent)]` is used.
                    let ty = type_registry::type_to_ident(elem).unwrap();
                    let ident = get_ident_from_pat(&t.pat);
                    if type_registry.is_newtype(ty) {
                        stmts.push(parse_quote! {
                            let #ident = &#ident.0;
                        });
                    }
                }
                _ => (),
            }
        }
    }

    stmts.extend(func.block.stmts.clone());
    func.block.stmts = stmts;
}
