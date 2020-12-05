//! Object Identifier (OID) constants with heapless `no_std` support.
//!
//! This crate supports creating [`ObjectIdentifier`] constants with
//! compile-time checks on validity.
//!
//! It has full `no_std` support with no dependencies on a heap/liballoc and
//! stores OID values as static data.
//!
//! The [`ObjectIdentifier`] type can also be used for runtime modeling of OIDs
//! (e.g. when parsing messages), with the goal of easily comparing them to
//! constant values.
//!
//! # About OIDs
//!
//! Object Identifiers, a.k.a. OIDs, are an International Telecommunications Union (ITU) and
//! ISO/IEC standard for naming any object, concept, or "thing" with a globally unambiguous
//! persistent name.
//!
//! See also: <https://en.wikipedia.org/wiki/Object_identifier>
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.46** at a minimum.

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/const-oid/0.3.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(any(feature = "std", test))]
extern crate std;

use core::{convert::TryFrom, fmt, str::FromStr};

/// Maximum number of nodes in an OID.
///
/// Note: this is specialized to RustCrypto use cases for now.
pub const MAX_NODES: usize = 10;

/// Maximum value of the first node in an OID
const FIRST_NODE_MAX: u32 = 2;

/// Maximum value of the second node in an OID
const SECOND_NODE_MAX: u32 = 39;

/// Error type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OID error")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Object identifier (OID)
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ObjectIdentifier {
    /// Nodes in this OID
    nodes: [u32; MAX_NODES],

    /// Number of nodes in this OID
    length: usize,
}

/// Constant panicking assertion.
// TODO(tarcieri): use const panic when stable.
// See: https://github.com/rust-lang/rust/issues/51999
macro_rules! const_assert {
    ($bool:expr, $msg:expr) => {
        [$msg][!$bool as usize]
    };
}

impl ObjectIdentifier {
    /// Create an OID from a slice of integers.
    ///
    /// # Panics
    ///
    /// To enable `const fn` usage and work around current limitations thereof,
    /// this method panics in the event the OID is malformed.
    ///
    /// For that reason this method is not recommended except for use in
    /// constants (where it will generate a compiler error instead).
    /// To parse an OID from a `&[u32]` slice without panicking on error,
    /// use [`TryFrom<&[u32]>`][1] instead.
    ///
    /// In order for an OID to be valid, it must meet the following criteria:
    ///
    /// - The OID MUST have at least 3 nodes
    /// - The OID MUST NOT have more nodes than the [`MAX_NODES`] constant
    /// - The first node MUST be within the range 0-2
    /// - The second node MUST be within the range 0-39
    ///
    /// [1]: ./struct.ObjectIdentifier.html#impl-TryFrom%3C%26%27_%20%5Bu32%5D%3E
    pub const fn new(nodes: &[u32]) -> Self {
        const_assert!(nodes.len() >= 3, "OID too short (minimum 3 nodes)");
        const_assert!(
            nodes.len() <= MAX_NODES,
            "OID too long (internal limit reached)"
        );
        const_assert!(
            nodes[0] <= FIRST_NODE_MAX,
            "invalid first node (must be 0-2)"
        );
        const_assert!(
            nodes[1] <= SECOND_NODE_MAX,
            "invalid second node (must be 0-39)"
        );

        // TODO(tarcieri): use `const_mut_ref` when stable.
        // See: <https://github.com/rust-lang/rust/issues/57349>
        #[rustfmt::skip]
        let n = match nodes.len() {
            3 => [
                nodes[0], nodes[1], nodes[2], 0, 0,
                0, 0, 0, 0, 0
            ],
            4 => [
                nodes[0], nodes[1], nodes[2], nodes[3], 0,
                0, 0, 0, 0, 0
            ],
            5 => [
                nodes[0], nodes[1], nodes[2], nodes[3], nodes[4],
                0, 0, 0, 0, 0,
            ],
            6 => [
                nodes[0], nodes[1], nodes[2], nodes[3], nodes[4],
                nodes[5], 0, 0, 0, 0,
            ],
            7 => [
                nodes[0], nodes[1], nodes[2], nodes[3], nodes[4],
                nodes[5], nodes[6], 0, 0, 0,
            ],
            8 => [
                nodes[0], nodes[1], nodes[2], nodes[3], nodes[4],
                nodes[5], nodes[6], nodes[7], 0, 0,
            ],
            9 => [
                nodes[0], nodes[1], nodes[2], nodes[3], nodes[4],
                nodes[5], nodes[6], nodes[7], nodes[8], 0,
            ],
            10 => [
                nodes[0], nodes[1], nodes[2], nodes[3], nodes[4],
                nodes[5], nodes[6], nodes[7], nodes[8], nodes[9],
            ],
            _ => [0u32; MAX_NODES], // Checks above prevent this case, but makes Miri happy
        };

        Self {
            nodes: n,
            length: nodes.len(),
        }
    }

    /// Parse an OID from from its BER/DER encoding.
    pub fn from_ber(mut bytes: &[u8]) -> Result<Self, Error> {
        let octet = parse_byte(&mut bytes)?;

        let mut nodes = [0u32; MAX_NODES];
        nodes[0] = (octet / (SECOND_NODE_MAX as u8 + 1)) as u32;
        nodes[1] = (octet % (SECOND_NODE_MAX as u8 + 1)) as u32;

        let mut length = 2;

        while !bytes.is_empty() {
            nodes[length] = parse_base128(&mut bytes)?;
            length += 1;

            if length > MAX_NODES {
                return Err(Error);
            }
        }

        if length < 3 {
            return Err(Error);
        }

        validate_nodes(nodes)?;
        Ok(Self { nodes, length })
    }
}

impl AsRef<[u32]> for ObjectIdentifier {
    fn as_ref(&self) -> &[u32] {
        &self.nodes[..self.length]
    }
}

impl FromStr for ObjectIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let mut nodes = [0u32; MAX_NODES];
        let mut length = 0;

        for (i, n) in s.split('.').enumerate() {
            if i + 1 == MAX_NODES {
                return Err(Error);
            }

            nodes[i] = n.parse().map_err(|_| Error)?;
            length += 1;
        }

        validate_nodes(nodes)?;
        Ok(Self { nodes, length })
    }
}

impl TryFrom<&[u32]> for ObjectIdentifier {
    type Error = Error;

    fn try_from(nodes: &[u32]) -> Result<Self, Error> {
        if nodes.len() < 3 || nodes.len() > MAX_NODES {
            return Err(Error);
        }

        let mut nodes_arr = [0u32; MAX_NODES];
        nodes_arr[..nodes.len()].copy_from_slice(nodes);
        validate_nodes(nodes_arr)?;

        Ok(Self {
            nodes: nodes_arr,
            length: nodes.len(),
        })
    }
}

impl fmt::Debug for ObjectIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectIdentifier({})", self)
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

/// Run validations on a node array
fn validate_nodes(nodes: [u32; MAX_NODES]) -> Result<(), Error> {
    if nodes[0] > FIRST_NODE_MAX {
        return Err(Error);
    }

    if nodes[1] > SECOND_NODE_MAX {
        return Err(Error);
    }

    Ok(())
}

/// Parse a base 128 (big endian) integer from a bytestring
fn parse_base128(bytes: &mut &[u8]) -> Result<u32, Error> {
    let mut result = 0;
    let mut shift = 0;

    loop {
        let byte = parse_byte(bytes)?;

        if shift == 28 && byte & 0b11110000 != 0 {
            // Overflow
            return Err(Error);
        }

        result = result << 7 | (byte & 0b1111111) as u32;

        if byte & 0b10000000 == 0 {
            return Ok(result);
        }

        shift += 7;
    }
}

/// Parse a single byte from a slice
fn parse_byte(bytes: &mut &[u8]) -> Result<u8, Error> {
    let byte = *bytes.get(0).ok_or(Error)?;
    *bytes = &bytes[1..];
    Ok(byte)
}

#[cfg(test)]
mod tests {
    use super::ObjectIdentifier;
    use std::{convert::TryFrom, string::ToString};

    /// Example OID value
    const EXAMPLE_OID: ObjectIdentifier = ObjectIdentifier::new(&[1, 2, 840, 10045, 2, 1]);

    /// Example OID encoded as ASN.1 BER/DER
    const EXAMPLE_OID_BER: &[u8] = &[42, 134, 72, 206, 61, 2, 1];

    /// Example OID as a string
    const EXAMPLE_OID_STRING: &str = "1.2.840.10045.2.1";

    #[test]
    fn display() {
        let oid = EXAMPLE_OID.to_string();
        assert_eq!(oid, EXAMPLE_OID_STRING);
    }

    #[test]
    fn from_ber() {
        let oid = ObjectIdentifier::from_ber(EXAMPLE_OID_BER).unwrap();
        assert_eq!(oid, EXAMPLE_OID);

        // Empty
        assert!(ObjectIdentifier::from_ber(&[]).is_err());

        // Truncated
        assert!(ObjectIdentifier::from_ber(&[42]).is_err());
        assert!(ObjectIdentifier::from_ber(&[42, 134]).is_err());
    }

    #[test]
    fn from_str() {
        let oid = EXAMPLE_OID_STRING.parse::<ObjectIdentifier>().unwrap();
        assert_eq!(oid, EXAMPLE_OID);

        // Truncated
        assert!("1.2.840.10045.2.".parse::<ObjectIdentifier>().is_err());

        // Invalid first node
        assert!("3.2.840.10045.2.1".parse::<ObjectIdentifier>().is_err());

        // Invalid second node
        assert!("1.40.840.10045.2.1".parse::<ObjectIdentifier>().is_err());
    }

    #[test]
    fn try_from_u32_slice() {
        let oid = ObjectIdentifier::try_from([1, 2, 840, 10045, 2, 1].as_ref()).unwrap();
        assert_eq!(EXAMPLE_OID, oid);

        // Truncated
        assert!(ObjectIdentifier::try_from([1, 2].as_ref()).is_err());

        // Invalid first node
        assert!(ObjectIdentifier::try_from([3, 2, 840, 10045, 3, 1, 7].as_ref()).is_err());

        // Invalid second node
        assert!(ObjectIdentifier::try_from([1, 40, 840, 10045, 3, 1, 7].as_ref()).is_err());
    }

    #[test]
    #[should_panic]
    fn truncated_oid() {
        ObjectIdentifier::new(&[1, 2]);
    }

    #[test]
    #[should_panic]
    fn invalid_first_node() {
        ObjectIdentifier::new(&[3, 2, 840, 10045, 3, 1, 7]);
    }

    #[test]
    #[should_panic]
    fn invalid_second_node() {
        ObjectIdentifier::new(&[1, 40, 840, 10045, 3, 1, 7]);
    }
}
