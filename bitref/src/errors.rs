//! Error types.

use core::fmt::{self, Display};

/// Index is out-of-bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IndexOutOfBounds;

impl core::error::Error for IndexOutOfBounds {}

impl Display for IndexOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("index out of bounds")
    }
}
