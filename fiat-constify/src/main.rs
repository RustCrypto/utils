//! Postprocessor for fiat-crypto generated field implementations which rewrites
//! them as `const fn`.
//!
//! Usage: fiat-constify /path/to/field_impl.rs

#![allow(clippy::single_match, clippy::new_without_default)]

mod outputs;
mod type_registry;

use outputs::Outputs;
use proc_macro2::{Punct, Spacing, Span};
use quote::{quote, TokenStreamExt};
use std::{collections::BTreeMap as Map, env, fs, ops::Deref};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Const, Eq, Let, Paren, Semi},
    Expr, ExprCall, ExprPath, ExprReference, Fields, FnArg, Ident, Item, ItemFn, Local, LocalInit,
    Meta, Pat, PatIdent, PatTuple, Path, Stmt, TypeReference,
};
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
            clippy::unnecessary_cast,
            dead_code,
            rustdoc::broken_intra_doc_links,
            unused_assignments,
            unused_mut,
            unused_variables
        )]
    });

    let mut type_registry = TypeRegistry::new();

    // Iterate over functions, transforming them into `const fn`
    let mut const_deref = Vec::new();
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

                let ident = &ty.ident;
                if let Fields::Unnamed(unnamed) = &ty.fields {
                    if let Some(unit) = unnamed.unnamed.first() {
                        let unit_ty = &unit.ty;
                        const_deref.push(parse_quote! {
                            impl #ident {
                                #[inline]
                                pub const fn as_inner(&self) -> &#unit_ty {
                                    &self.0
                                }

                                #[inline]
                                pub const fn into_inner(self) -> #unit_ty {
                                    self.0
                                }
                            }
                        });
                    }
                }

                type_registry.add_new_type(ty)
            }
            _ => (),
        }
    }
    ast.items.extend_from_slice(&const_deref);

    println!("//! fiat-crypto output postprocessed by fiat-constify: <https://github.com/rustcrypto/utils>");
    println!("{}", prettyplease::unparse(&ast));
    Ok(())
}

/// Get an `Ident` from a `Pat::Ident`.
fn get_ident_from_pat(pat: &Pat) -> Ident {
    match pat {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        other => panic!("unexpected `Pat`: {:?} (expecting `Pat::Ident`)", other),
    }
}

/// Rewrite a fiat-crypto generated `fn` as a `const fn`, making the necessary
/// transformations to the code in order for it to work in that context.
fn rewrite_fn_as_const(func: &mut ItemFn, type_registry: &TypeRegistry) {
    // Mark function as being `const fn`.
    func.sig.constness = Some(Const::default());

    // Transform mutable arguments into return values.
    let mut inputs = Punctuated::new();
    let mut outputs = Outputs::new(type_registry);
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
                    outputs.add(get_ident_from_pat(&t.pat), elem.deref().clone());
                    continue;
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
                    if outputs.type_registry().is_new_type(ty) {
                        stmts.push(parse_quote! {
                            let #ident = #ident.as_inner();
                        });
                    }
                }
                _ => (),
            }
        }

        // If the argument wasn't a mutable reference, add it as an input.
        inputs.push(arg.clone());
    }

    // Replace inputs with ones where the mutable references have been filtered out
    func.sig.inputs = inputs;
    func.sig.output = outputs.to_return_type();
    stmts.extend(rewrite_fn_body(&func.block.stmts, &outputs));
    func.block.stmts = stmts;
}

/// Rewrite the function body, adding let bindings with `Default::default()`
/// values for outputs, removing mutable references, and adding a return
/// value/tuple.
fn rewrite_fn_body(stmts: &[Stmt], outputs: &Outputs) -> Vec<Stmt> {
    let mut ident_assignments: Map<&Ident, Vec<&Expr>> = Map::new();
    let mut rewritten = Vec::new();

    for stmt in stmts {
        if let Stmt::Expr(Expr::Assign(assignment), Some(_)) = stmt {
            let lhs_path = match assignment.left.as_ref() {
                Expr::Unary(lhs) => {
                    if let Expr::Path(exprpath) = lhs.expr.as_ref() {
                        Some(exprpath)
                    } else {
                        panic!("All unary exprpaths should have the LHS as the path");
                    }
                }
                Expr::Index(lhs) => {
                    if let Expr::Path(exprpath) = lhs.expr.as_ref() {
                        Some(exprpath)
                    } else {
                        panic!("All unary exprpaths should have the LHS as the path");
                    }
                }
                Expr::Call(expr) => {
                    rewritten.push(Stmt::Local(rewrite_fn_call(expr.clone())));
                    None
                }
                _ => None,
            };
            if let Some(lhs_path) = lhs_path {
                ident_assignments
                    .entry(Path::get_ident(&lhs_path.path).unwrap())
                    .or_default()
                    .push(&assignment.right);
            }
        } else if let Stmt::Expr(Expr::Call(expr), Some(_)) = stmt {
            rewritten.push(Stmt::Local(rewrite_fn_call(expr.clone())));
        } else if let Stmt::Local(Local {
            pat: Pat::Type(pat),
            ..
        }) = stmt
        {
            let unboxed = pat.pat.as_ref();
            if let Pat::Ident(PatIdent {
                mutability: Some(_),
                ..
            }) = unboxed
            {
                // This is a mut var, in the case of fiat-crypto transformation dead code
            } else {
                rewritten.push(stmt.clone());
            }
        } else {
            rewritten.push(stmt.clone());
        }
    }

    let mut asts = Vec::new();
    for (ident, ty) in outputs.ident_type_pairs() {
        let value = ident_assignments.get(ident).unwrap();
        let type_prefix = match type_registry::type_to_ident(ty) {
            Some(ident) if outputs.type_registry().is_new_type(ident) => Some(ty),
            _ => None,
        };

        let ast = match (type_prefix, value.len()) {
            (None, 1) => {
                let first = value.first().unwrap();
                quote!(#first)
            }
            (Some(prefix), 1) => {
                let first = value.first().unwrap();
                quote!(#prefix(#first))
            }

            (None, _) => {
                quote!([#(#value),*])
            }
            (Some(prefix), _) => {
                quote!(#prefix([#(#value),*]))
            }
        };
        asts.push(ast);
    }

    let expr: Expr = parse_quote! {
        (#(#asts),*)
    };

    rewritten.push(Stmt::Expr(expr, None));
    rewritten
}

/// Rewrite a function call, removing the mutable reference arguments and
/// let-binding return values for them instead.
fn rewrite_fn_call(mut call: ExprCall) -> Local {
    let mut args = Punctuated::new();
    let mut output = Punctuated::new();

    for arg in &call.args {
        if let Expr::Reference(ExprReference {
            mutability: Some(_),
            expr,
            ..
        }) = arg
        {
            match expr.deref() {
                Expr::Path(ExprPath {
                    path: Path { segments, .. },
                    ..
                }) => {
                    assert_eq!(segments.len(), 1, "expected only one segment in fn arg");
                    let ident = segments.first().unwrap().ident.clone();

                    output.push(Pat::Ident(PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: None,
                        ident,
                        subpat: None,
                    }));
                }
                other => panic!("unexpected expr in fn arg: {:?}", other),
            }

            continue;
        }

        args.push(arg.clone());
    }

    // Overwrite call arguments with the ones that aren't mutable references
    call.args = args;

    let pat = Pat::Tuple(PatTuple {
        attrs: Vec::new(),
        paren_token: Paren::default(),
        elems: output,
    });

    Local {
        attrs: Vec::new(),
        let_token: Let::default(),
        pat,
        init: Some(LocalInit {
            eq_token: Eq::default(),
            expr: Box::new(Expr::Call(call)),
            diverge: None,
        }),
        semi_token: Semi::default(),
    }
}
