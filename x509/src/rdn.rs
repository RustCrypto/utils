//! Relative Distinguished Names

use crate::{AttributeTypeAndValue, Set};

/// Relative Distinguished Name
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct RelativeDistinguishedName<'a>(Set<AttributeTypeAndValue<'a>>);
