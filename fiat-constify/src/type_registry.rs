//! Keeps track of which type is aliasing an existing type and which are new types.
//!  This is useful because we only need to generate the return type prefixes for new types
//!
use std::collections::BTreeMap as Map;
use syn::{Ident, ItemStruct, ItemType, Path};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Alias,
    NewType,
}

/// Registry of types defined by the module being processed.
pub struct TypeRegistry(Map<Ident, Type>);

impl TypeRegistry {
    /// Create a new type registry.
    pub fn new() -> Self {
        Self(Map::new())
    }

    /// Add a type which is a new type to the type registry.
    pub fn add_new_type(&mut self, item_struct: &ItemStruct) {
        if self
            .0
            .insert(item_struct.ident.clone(), Type::NewType)
            .is_some()
        {
            panic!("duplicate type name: {}", &item_struct.ident);
        }
    }

    /// Add a type which is a type alias
    pub fn add_type_alias(&mut self, item_type: &ItemType) {
        if self
            .0
            .insert(item_type.ident.clone(), Type::Alias)
            .is_some()
        {
            panic!("duplicate type name: {}", &item_type.ident);
        }
    }

    /// Get the [`Type`] which the identifier is.
    ///
    /// Returns `None` whe ident can't be found.
    pub fn get(&self, ident: &Ident) -> Option<Type> {
        self.0.get(ident).copied()
    }

    pub fn is_new_type(&self, ident: &syn::Ident) -> bool {
        let mut included = false;
        if matches!(self.get(ident), Some(Type::NewType)) {
            included = true;
        }

        included
    }
}

#[inline]
pub fn type_to_ident(ty: &syn::Type) -> Option<&Ident> {
    if let syn::Type::Path(path) = ty {
        return Path::get_ident(&path.path);
    }

    None
}
