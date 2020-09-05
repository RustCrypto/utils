//! Object Identifier (OID) constants with heapless `no_std` support.
//!
//! This crate supports creating [`ObjectIdentifier`] constants with
//! compile-time checks on validity.
//!
//! It has full `no_std` support with no dependencies on a heap/liballoc and
//! stores OID values as static data.
//!
//! # About OIDs
//!
//! Object Identifiers, a.k.a. OIDs, are an International Telecommunications Union (ITU) and
//! ISO/IEC standard for naming any object, concept, or "thing" with a globally unambiguous
//! persistent name.
//!
//! See also: https://en.wikipedia.org/wiki/Object_identifier
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.46** at a minimum.

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/const-oid/0.2.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(test)]
extern crate std;

use core::fmt;

/// Object identifier (OID)
pub struct ObjectIdentifier {
    /// Nodes in this OID
    nodes: &'static [u32],
}

impl ObjectIdentifier {
    /// Create a new OID
    #[allow(clippy::no_effect)]
    pub const fn new(nodes: &'static [u32]) -> Self {
        // TODO(tarcieri): replace this with const panic when OIDs are invalid
        // See: https://github.com/rust-lang/rust/issues/51999
        let mut is_invalid = nodes.len() <= 2;

        match nodes[0] {
            0..=2 => (),
            _ => is_invalid = true,
        }

        match nodes[1] {
            0..=39 => {}
            _ => is_invalid = true,
        }

        // TODO(tarcieri): better error message
        ["invalid OID"][is_invalid as usize];

        Self { nodes }
    }
}

impl AsRef<[u32]> for ObjectIdentifier {
    fn as_ref(&self) -> &[u32] {
        self.nodes
    }
}

impl fmt::Display for ObjectIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, node) in self.as_ref().iter().enumerate() {
            write!(f, "{}", node)?;

            if i < self.as_ref().len() - 1 {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ObjectIdentifier;
    use std::string::ToString;

    const EXAMPLE_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 10045, 3, 1, 7]);

    #[test]
    fn display_test() {
        let oid = EXAMPLE_OID.to_string();
        assert_eq!(oid, "1.2.840.10045.3.1.7");
    }

    #[test]
    #[should_panic]
    fn truncated_oid_test() {
        ObjectIdentifier::new(&[1, 2]);
    }

    #[test]
    #[should_panic]
    fn invalid_first_node_test() {
        ObjectIdentifier::new(&[3, 2, 840, 10045, 3, 1, 7]);
    }

    #[test]
    #[should_panic]
    fn invalid_second_node_test() {
        ObjectIdentifier::new(&[1, 40, 840, 10045, 3, 1, 7]);
    }
}
