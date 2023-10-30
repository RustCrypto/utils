use crate::type_registry::TypeRegistry;
use syn::{
    punctuated::Punctuated,
    token::{Paren, RArrow},
    Ident, ReturnType, Type, TypeTuple,
};

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
pub struct Outputs<'a> {
    type_registry: &'a TypeRegistry,
    outputs: Vec<(Ident, Type)>,
}

impl<'a> Outputs<'a> {
    #[inline]
    /// Create new output storage.
    pub fn new(type_registry: &'a TypeRegistry) -> Self {
        Self {
            type_registry,
            outputs: Vec::new(),
        }
    }

    #[inline]
    pub fn type_registry(&self) -> &TypeRegistry {
        self.type_registry
    }

    #[inline]
    pub fn ident_type_pairs(&self) -> impl Iterator<Item = &(Ident, Type)> + ExactSizeIterator {
        self.outputs.iter()
    }

    #[inline]
    fn types(&self) -> impl Iterator<Item = &Type> + ExactSizeIterator {
        self.outputs.iter().map(|(_, ty)| ty)
    }

    /// Add an output variable with the given name and type.
    #[inline]
    pub fn add(&mut self, name: Ident, ty: Type) {
        self.outputs.push((name.clone(), ty));
    }

    /// Finish annotating outputs, updating the provided `Signature`.
    pub fn to_return_type(&self) -> ReturnType {
        let rarrow = RArrow::default();

        let ret = match self.outputs.len() {
            0 => panic!("expected at least one output"),
            1 => self.types().next().unwrap().clone(),
            _ => {
                let mut elems = Punctuated::new();

                for ty in self.types() {
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
}
