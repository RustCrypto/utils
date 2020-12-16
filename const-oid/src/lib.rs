//! Const-friendly X.660 Object Identifier (OID) library with support for
//! heapless `no_std` (i.e. embedded) environments.
//!
//! # About OIDs
//!
//! Object Identifiers, a.k.a. OIDs, are an International Telecommunications
//! Union (ITU) and ISO/IEC standard for naming any object, concept, or "thing"
//! with a globally unambiguous persistent name.
//!
//! OIDS are defined in the ITU's [X.660] standard.
//!
//! The following is an example of an OID, in this case identifying the
//! `rsaEncryption` algorithm:
//!
//! ```text
//! 1.2.840.113549.1.1.1
//! ```
//!
//! For more information, see: <https://en.wikipedia.org/wiki/Object_identifier>
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.46** at a minimum.
//!
//! Minimum supported Rust version may be changed in the future, but it will be
//! accompanied with a minor version bump.
//!
//! [X.660]: https://www.itu.int/rec/T-REC-X.660

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/const-oid/0.3.5"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

use core::{convert::TryFrom, fmt, str::FromStr};

/// Maximum number of arcs in an OID.
///
/// Note: this limit is not a part of the OID standard, but represents an
/// internal size constraint of this library.
pub const MAX_ARCS: usize = 10;

/// Maximum value of the first arc in an OID
const FIRST_ARC_MAX: u32 = 2;

/// Maximum value of the second arc in an OID
const SECOND_ARC_MAX: u32 = 39;

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

/// Result type
pub type Result<T> = core::result::Result<T, Error>;

/// Object identifier (OID)
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ObjectIdentifier {
    /// Arcs in this OID
    arcs: [u32; MAX_ARCS],

    /// Number of arcs in this OID
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

#[allow(clippy::len_without_is_empty)]
impl ObjectIdentifier {
    /// Create an [`ObjectIdentifier`] from a slice of integers, where each
    /// integer represents an "arc" (a.k.a. node) in the OID.
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
    /// - The OID MUST have at least 3 arcs
    /// - The OID MUST NOT have more arcs than the [`MAX_ARCS`] constant
    /// - The first arc MUST be within the range 0-2
    /// - The second arc MUST be within the range 0-39
    ///
    /// [1]: ./struct.ObjectIdentifier.html#impl-TryFrom%3C%26%27_%20%5Bu32%5D%3E
    pub const fn new(arcs: &[u32]) -> Self {
        const_assert!(arcs.len() >= 3, "OID too short (minimum 3 arcs)");
        const_assert!(
            arcs.len() <= MAX_ARCS,
            "OID too long (internal limit reached)"
        );
        const_assert!(arcs[0] <= FIRST_ARC_MAX, "invalid first arc (must be 0-2)");
        const_assert!(
            arcs[1] <= SECOND_ARC_MAX,
            "invalid second arc (must be 0-39)"
        );

        // TODO(tarcieri): use `const_mut_ref` when stable.
        // See: <https://github.com/rust-lang/rust/issues/57349>
        #[rustfmt::skip]
        let n = match arcs.len() {
            3 => [
                arcs[0], arcs[1], arcs[2], 0, 0,
                0, 0, 0, 0, 0
            ],
            4 => [
                arcs[0], arcs[1], arcs[2], arcs[3], 0,
                0, 0, 0, 0, 0
            ],
            5 => [
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4],
                0, 0, 0, 0, 0,
            ],
            6 => [
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4],
                arcs[5], 0, 0, 0, 0,
            ],
            7 => [
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4],
                arcs[5], arcs[6], 0, 0, 0,
            ],
            8 => [
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4],
                arcs[5], arcs[6], arcs[7], 0, 0,
            ],
            9 => [
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4],
                arcs[5], arcs[6], arcs[7], arcs[8], 0,
            ],
            10 => [
                arcs[0], arcs[1], arcs[2], arcs[3], arcs[4],
                arcs[5], arcs[6], arcs[7], arcs[8], arcs[9],
            ],
            _ => [0u32; MAX_ARCS], // Checks above prevent this case, but makes Miri happy
        };

        Self {
            arcs: n,
            length: arcs.len(),
        }
    }

    /// Iterate over the arcs (a.k.a. nodes) in an [`ObjectIdentifier`].
    ///
    /// Returns [`Arcs`], an iterator over `u32` values representing the value
    /// of each arc/node.
    pub fn arcs(&self) -> Arcs {
        Arcs {
            oid: *self,
            index: 0,
        }
    }

    /// Number of arcs in this [`ObjectIdentifier`].
    pub fn len(&self) -> usize {
        self.length
    }

    /// Parse an OID from from its BER/DER encoding.
    pub fn from_ber(mut bytes: &[u8]) -> Result<Self> {
        let octet = parse_byte(&mut bytes)?;

        let mut arcs = [0u32; MAX_ARCS];
        arcs[0] = (octet / (SECOND_ARC_MAX as u8 + 1)) as u32;
        arcs[1] = (octet % (SECOND_ARC_MAX as u8 + 1)) as u32;

        let mut length = 2;

        while !bytes.is_empty() {
            arcs[length] = parse_base128(&mut bytes)?;
            length += 1;

            if length > MAX_ARCS {
                return Err(Error);
            }
        }

        if length < 3 {
            return Err(Error);
        }

        validate_arcs(arcs)?;
        Ok(Self { arcs, length })
    }

    /// Get the length of this OID when serialized as ASN.1 BER.
    pub fn ber_len(&self) -> usize {
        self.arcs[2..self.length]
            .iter()
            .fold(1, |sum, n| sum + base128_len(*n))
    }

    /// Write the BER encoding of this OID into the given slice, returning
    /// a new slice containing the written data.
    pub fn write_ber<'a>(&self, bytes: &'a mut [u8]) -> Result<&'a [u8]> {
        if bytes.is_empty() {
            return Err(Error);
        }

        bytes[0] = (self.arcs[0] * (SECOND_ARC_MAX + 1)) as u8 | self.arcs[1] as u8;

        let mut offset = 1;

        for &arc in &self.arcs[2..self.length] {
            offset += write_base128(&mut bytes[offset..], arc)?;
        }

        Ok(&bytes[..offset])
    }

    /// Serialize this OID as ASN.1 BER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_ber(&self) -> alloc::vec::Vec<u8> {
        let mut output = vec![0u8; self.ber_len()];
        self.write_ber(&mut output)
            .expect("incorrectly sized buffer");
        output
    }
}

impl FromStr for ObjectIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut arcs = [0u32; MAX_ARCS];
        let mut length = 0;

        for (i, n) in s.split('.').enumerate() {
            if i + 1 == MAX_ARCS {
                return Err(Error);
            }

            arcs[i] = n.parse().map_err(|_| Error)?;
            length += 1;
        }

        validate_arcs(arcs)?;
        Ok(Self { arcs, length })
    }
}

impl TryFrom<&[u32]> for ObjectIdentifier {
    type Error = Error;

    fn try_from(arcs: &[u32]) -> Result<Self> {
        if arcs.len() < 3 || arcs.len() > MAX_ARCS {
            return Err(Error);
        }

        let mut arcs_arr = [0u32; MAX_ARCS];
        arcs_arr[..arcs.len()].copy_from_slice(arcs);
        validate_arcs(arcs_arr)?;

        Ok(Self {
            arcs: arcs_arr,
            length: arcs.len(),
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
        for (i, arc) in self.arcs().enumerate() {
            write!(f, "{}", arc)?;

            if i < self.len() - 1 {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}

/// [`Iterator`] over arcs (a.k.a. nodes) in an [`ObjectIdentifier`].
///
/// This iterates over all arcs in an OID, including the root.
pub struct Arcs {
    /// OID we're iterating over
    oid: ObjectIdentifier,

    /// Current arc
    index: usize,
}

impl Iterator for Arcs {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.index < self.oid.length {
            let arc = self.oid.arcs[self.index];
            self.index = self.index.checked_add(1).unwrap();
            Some(arc)
        } else {
            None
        }
    }
}

/// Run validations on a arc array
fn validate_arcs(arcs: [u32; MAX_ARCS]) -> Result<()> {
    if arcs[0] > FIRST_ARC_MAX {
        return Err(Error);
    }

    if arcs[1] > SECOND_ARC_MAX {
        return Err(Error);
    }

    Ok(())
}

/// Parse a single byte from a slice
fn parse_byte(bytes: &mut &[u8]) -> Result<u8> {
    let byte = *bytes.get(0).ok_or(Error)?;
    *bytes = &bytes[1..];
    Ok(byte)
}

/// Parse a base 128 (big endian) integer from a bytestring
fn parse_base128(bytes: &mut &[u8]) -> Result<u32> {
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

/// Write the given unsigned integer in base 128
fn write_base128(bytes: &mut [u8], mut n: u32) -> Result<usize> {
    let nbytes = base128_len(n);
    let mut i = nbytes.checked_sub(1).expect("length underflow");
    let mut mask = 0;

    while n > 0x80 {
        let byte = bytes.get_mut(i).ok_or(Error)?;
        *byte = (n & 0b1111111 | mask) as u8;
        n >>= 7;
        i = i.checked_sub(1).unwrap();
        mask = 0b10000000;
    }

    *bytes.get_mut(0).unwrap() = (n | mask) as u8;
    Ok(nbytes)
}

/// Compute the length of a value when encoded in base 128
fn base128_len(n: u32) -> usize {
    match n {
        0..=0x7f => 1usize,
        0x80..=0x3fff => 2,
        0x4000..=0x1fffff => 3,
        0x200000..=0x1fffffff => 4,
        _ => 5,
    }
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

        // Invalid first arc
        assert!("3.2.840.10045.2.1".parse::<ObjectIdentifier>().is_err());

        // Invalid second arc
        assert!("1.40.840.10045.2.1".parse::<ObjectIdentifier>().is_err());
    }

    #[test]
    fn try_from_u32_slice() {
        let oid = ObjectIdentifier::try_from([1, 2, 840, 10045, 2, 1].as_ref()).unwrap();
        assert_eq!(EXAMPLE_OID, oid);

        // Truncated
        assert!(ObjectIdentifier::try_from([1, 2].as_ref()).is_err());

        // Invalid first arc
        assert!(ObjectIdentifier::try_from([3, 2, 840, 10045, 3, 1, 7].as_ref()).is_err());

        // Invalid second arc
        assert!(ObjectIdentifier::try_from([1, 40, 840, 10045, 3, 1, 7].as_ref()).is_err());
    }

    #[test]
    fn write_ber() {
        let mut buffer = [0u8; 16];
        let slice = EXAMPLE_OID.write_ber(&mut buffer).unwrap();
        assert_eq!(slice, EXAMPLE_OID_BER);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn to_ber() {
        assert_eq!(EXAMPLE_OID.to_ber(), EXAMPLE_OID_BER);
    }

    #[test]
    #[should_panic]
    fn truncated_oid() {
        ObjectIdentifier::new(&[1, 2]);
    }

    #[test]
    #[should_panic]
    fn invalid_first_arc() {
        ObjectIdentifier::new(&[3, 2, 840, 10045, 3, 1, 7]);
    }

    #[test]
    #[should_panic]
    fn invalid_second_arc() {
        ObjectIdentifier::new(&[1, 40, 840, 10045, 3, 1, 7]);
    }
}
