//! Postprocessor for fiat-crypto generated field implementations which rewrites
//! them as `const fn`.
//!
//! Usage: fiat-constify /path/to/field_impl.rs

#![allow(clippy::single_match, clippy::new_without_default)]

use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use std::{collections::BTreeMap as Map, env, fs, ops::Deref};
use syn::{
    punctuated::Punctuated,
    token::{Brace, Bracket, Colon, Const, Eq, Let, Mut, Not, Paren, Pound, RArrow, Semi},
    AttrStyle, Attribute, Block, Expr, ExprAssign, ExprCall, ExprLit, ExprPath, ExprReference,
    ExprRepeat, ExprTuple, FnArg, Ident, Item, ItemFn, ItemType, Lit, LitInt, Local, LocalInit,
    MacroDelimiter, Meta, MetaList, Pat, PatIdent, PatTuple, PatType, Path, PathArguments,
    PathSegment, ReturnType, Stmt, Type, TypeArray, TypePath, TypeReference, TypeTuple, UnOp,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();

    if args.len() != 2 {
        panic!("Usage: fiat-constify /path/to/field_impl.rs")
    }

    let code = fs::read_to_string(&args[1])?;
    let mut ast = syn::parse_file(&code)?;

    // Add lint attributes
    ast.attrs.push(build_attribute(
        "allow",
        &[
            "clippy::identity_op",
            "clippy::unnecessary_cast",
            "dead_code",
            "rustdoc::broken_intra_doc_links",
            "unused_assignments",
            "unused_mut",
            "unused_variables",
        ],
    ));

    let mut type_registry = TypeRegistry::new();

    // Iterate over functions, transforming them into `const fn`
    for item in &mut ast.items {
        match item {
            Item::Fn(func) => rewrite_fn_as_const(func, &type_registry),
            Item::Type(ty) => type_registry.add(ty),
            _ => (),
        }
    }

    println!("#![doc = \" fiat-crypto output postprocessed by fiat-constify: <https://github.com/rustcrypto/utils>\"]");
    println!("{}", ast.into_token_stream());
    Ok(())
}

/// Build a toplevel attribute with the given name and comma-separated values.
fn build_attribute(name: &str, values: &[&str]) -> Attribute {
    let values = values
        .iter()
        .map(|value| build_path(value))
        .collect::<Vec<_>>();
    let path = build_path(name);
    let tokens = quote! { #(#values),* };
    let delimiter = MacroDelimiter::Paren(Paren::default());

    Attribute {
        pound_token: Pound::default(),
        style: AttrStyle::Inner(Not::default()),
        bracket_token: Bracket::default(),
        meta: Meta::List(MetaList {
            path,
            delimiter,
            tokens,
        }),
    }
}

/// Parse a path from a double-colon-delimited string.
fn build_path(path: &str) -> Path {
    let mut segments = Punctuated::new();

    for segment in path.split("::") {
        segments.push(PathSegment {
            ident: Ident::new(segment, Span::call_site()),
            arguments: PathArguments::None,
        });
    }

    Path {
        leading_colon: None,
        segments,
    }
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
    let mut outputs = Outputs::new();

    for arg in &func.sig.inputs {
        // Transform mutable function arguments into return values
        if let FnArg::Typed(t) = arg {
            match &*t.ty {
                Type::Reference(TypeReference {
                    mutability: Some(_), // look for mutable references
                    elem,
                    ..
                }) => {
                    outputs.add(get_ident_from_pat(&t.pat), elem.deref().clone());
                    continue;
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
    func.block = Box::new(rewrite_fn_body(&func.block.stmts, &outputs, type_registry));
}

/// Rewrite the function body, adding let bindings with `Default::default()`
/// values for outputs, removing mutable references, and adding a return
/// value/tuple.
fn rewrite_fn_body(statements: &[Stmt], outputs: &Outputs, registry: &TypeRegistry) -> Block {
    let mut stmts = Vec::new();

    stmts.extend(outputs.to_let_bindings(registry).into_iter());

    for stmt in statements {
        let mut stmt = stmt.clone();
        rewrite_fn_stmt(&mut stmt);
        stmts.push(stmt.clone());
    }

    stmts.push(outputs.to_return_value());

    Block {
        brace_token: Brace::default(),
        stmts,
    }
}

/// Rewrite an expression in the function body, transforming mutable reference
/// operations into value assignments.
fn rewrite_fn_stmt(stmt: &mut Stmt) {
    match stmt {
        Stmt::Expr(expr, Some(_)) => match expr {
            Expr::Assign(ExprAssign { left, .. }) => match *left.clone() {
                Expr::Unary(unary) => {
                    // Remove deref since we're removing mutable references
                    if matches!(unary.op, UnOp::Deref(_)) {
                        *left = unary.expr;
                    }
                }
                _ => (),
            },
            Expr::Call(call) => {
                *stmt = Stmt::Local(rewrite_fn_call(call.clone()));
            }
            _ => (),
        },
        _ => (),
    }
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

/// Registry of types defined by the module being processed.
pub struct TypeRegistry(Map<Ident, Type>);

impl TypeRegistry {
    /// Create a new type registry.
    pub fn new() -> Self {
        Self(Map::new())
    }

    /// Add a type to the type registry.
    pub fn add(&mut self, item_type: &ItemType) {
        if self
            .0
            .insert(item_type.ident.clone(), item_type.ty.deref().clone())
            .is_some()
        {
            panic!("duplicate type name: {}", &item_type.ident);
        }
    }

    /// Get a type from the registry by its ident.
    pub fn get(&self, ident: &Ident) -> Option<&Type> {
        self.0.get(ident)
    }
}

/// Output values, which in regular `fiat-crypto` are passed as mutable references, e.g.:
///
/// ```
/// out1: &mut ..., out2: &mut ...
/// ```
///
/// This type stores the outputs and uses them to build the return type
/// (i.e. `Signature::output`), `let mut` bindings in place of the mutable
/// references, and a return value instead of using side effects to write to
/// mutable references.
#[derive(Debug)]
pub struct Outputs(Map<Ident, Type>);

impl Outputs {
    /// Create new output storage.
    pub fn new() -> Self {
        Self(Map::new())
    }

    /// Add an output variable with the given name and type.
    ///
    /// Panics if the name is duplicated.
    pub fn add(&mut self, name: Ident, ty: Type) {
        if self.0.insert(name.clone(), ty).is_some() {
            panic!("duplicate output name: {}", name);
        }
    }

    /// Generate `let mut outN: Ty = <zero>` bindings at the start
    /// of the function.
    pub fn to_let_bindings(&self, registry: &TypeRegistry) -> Vec<Stmt> {
        self.0
            .iter()
            .map(|(ident, ty)| {
                Stmt::Local(Local {
                    attrs: Vec::new(),
                    let_token: Let::default(),
                    pat: Pat::Type(PatType {
                        attrs: Vec::new(),
                        pat: Box::new(Pat::Ident(PatIdent {
                            attrs: Vec::new(),
                            by_ref: None,
                            mutability: Some(Mut::default()),
                            ident: ident.clone(),
                            subpat: None,
                        })),
                        colon_token: Colon::default(),
                        ty: Box::new(ty.clone()),
                    }),
                    init: Some(LocalInit {
                        eq_token: Eq::default(),
                        expr: Box::new(default_for(ty, registry)),
                        diverge: None,
                    }),
                    semi_token: Semi::default(),
                })
            })
            .collect()
    }

    /// Finish annotating outputs, updating the provided `Signature`.
    pub fn to_return_type(&self) -> ReturnType {
        let rarrow = RArrow::default();

        let ret = match self.0.len() {
            0 => panic!("expected at least one output"),
            1 => self.0.values().next().unwrap().clone(),
            _ => {
                let mut elems = Punctuated::new();

                for ty in self.0.values() {
                    elems.push(ty.clone());
                }

                Type::Tuple(TypeTuple {
                    paren_token: Paren::default(),
                    elems,
                })
            }
        };

        ReturnType::Type(rarrow, Box::new(ret))
    }

    /// Generate the return value for the statement as a tuple of the outputs.
    pub fn to_return_value(&self) -> Stmt {
        let mut elems = self.0.keys().map(|ident| {
            let mut segments = Punctuated::new();
            segments.push(PathSegment {
                ident: ident.clone(),
                arguments: PathArguments::None,
            });

            let path = Path {
                leading_colon: None,
                segments,
            };

            Expr::Path(ExprPath {
                attrs: Vec::new(),
                qself: None,
                path,
            })
        });

        if elems.len() == 1 {
            Stmt::Expr(elems.next().unwrap(), None)
        } else {
            Stmt::Expr(
                Expr::Tuple(ExprTuple {
                    attrs: Vec::new(),
                    paren_token: Paren::default(),
                    elems: elems.collect(),
                }),
                None,
            )
        }
    }
}

/// Get a default value for the given type.
fn default_for(ty: &Type, registry: &TypeRegistry) -> Expr {
    let zero = Expr::Lit(ExprLit {
        attrs: Vec::new(),
        lit: Lit::Int(LitInt::from(Literal::u8_unsuffixed(0))),
    });

    match ty {
        Type::Array(TypeArray { len, .. }) => Expr::Repeat(ExprRepeat {
            attrs: Vec::new(),
            bracket_token: Bracket::default(),
            expr: Box::new(zero),
            semi_token: Semi::default(),
            len: Box::new(len.clone()),
        }),
        Type::Path(TypePath { path, .. }) => {
            assert_eq!(
                path.segments.len(),
                1,
                "wasn't expecting multiple segments in path"
            );

            let ident = path.segments.first().unwrap().ident.clone();

            // Attempt to look up type in the registry
            if let Some(registry_ty) = registry.get(&ident) {
                // If we got a type from the registry, recurse
                default_for(registry_ty, registry)
            } else if matches!(ident.to_string().as_str(), "u8" | "u32" | "u64") {
                zero
            } else {
                panic!("unsupported type: {:?}", ty)
            }
        }
        _ => panic!("don't know how to build default value for {:?}", ty),
    }
}
