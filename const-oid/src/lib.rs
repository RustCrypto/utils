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
//! # Limits
//!
//! This library stores OIDs using a compact fixed-size layout and enforces
//! the following constraints on the number of arcs:
//!
//! - Minimum number of arcs: **3** (i.e. [`MIN_ARCS`])
//! - Maximum number of arcs: **12** (i.e. [`MAX_ARCS`])
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
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/const-oid/0.4.1"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

use core::{
    convert::TryFrom,
    fmt,
    str::{FromStr, Split},
};

/// Minimum number of arcs in an OID.
///
/// This library will refuse to parse OIDs with fewer than this number of arcs.
pub const MIN_ARCS: usize = 3;

/// Maximum number of arcs in an OID.
///
/// Note: this limit is not defined in OID standards, but instead represents an
/// internal size constraint of this library determined as an upper bound for
/// this library's intended use cases (i.e. [RustCrypto projects][1]).
///
/// It can potentially be raised as part of a breaking release if there is a
/// legitimate use case. If you have such a use case for increasing this limit
/// in practice, please [file a GitHub issue][2].
///
/// [1]: https://github.com/RustCrypto/
/// [2]: https://github.com/RustCrypto/utils/issues
pub const MAX_ARCS: usize = 12;

/// Maximum number of "lower" arcs, which does not include the first and second
/// arcs, which are stored as [`RootArcs`].
const MAX_LOWER_ARCS: usize = MAX_ARCS - 2;

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
    /// Byte containing the first and second arcs of this OID, stored
    /// separately from the others to minimize this type's size.
    root_arcs: RootArcs,

    /// Additional "lower" arcs beyond the first and second arcs, which are
    /// stored as [`RootArcs`].
    lower_arcs: LowerArcs,
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
        const_assert!(arcs.len() >= MIN_ARCS, "OID too short (minimum 3 arcs)");
        const_assert!(
            arcs.len() <= MAX_ARCS,
            "OID too long (internal limit reached)"
        );

        let first_arc = arcs[0];
        const_assert!(
            first_arc <= FIRST_ARC_MAX,
            "invalid first arc (must be 0-2)"
        );

        let second_arc = arcs[1];
        const_assert!(
            second_arc <= SECOND_ARC_MAX,
            "invalid second arc (must be 0-39)"
        );

        let root_arcs = RootArcs((first_arc * (SECOND_ARC_MAX + 1)) as u8 | second_arc as u8);

        // TODO(tarcieri): use `const_mut_ref` when stable.
        // See: <https://github.com/rust-lang/rust/issues/57349>
        #[rustfmt::skip]
        let lower_arcs = match arcs.len() {
            3 => [
                arcs[2], 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ],
            4 => [
                arcs[2], arcs[3], 0, 0, 0,
                0, 0, 0, 0, 0
            ],
            5 => [
                arcs[2], arcs[3], arcs[4], 0, 0,
                0, 0, 0, 0, 0
            ],
            6 => [
                arcs[2], arcs[3], arcs[4], arcs[5], 0,
                0, 0, 0, 0, 0
            ],
            7 => [
                arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
                0, 0, 0, 0, 0,
            ],
            8 => [
                arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
                arcs[7], 0, 0, 0, 0,
            ],
            9 => [
                arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
                arcs[7], arcs[8], 0, 0, 0,
            ],
            10 => [
                arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
                arcs[7], arcs[8], arcs[9], 0, 0,
            ],
            11 => [
                arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
                arcs[7], arcs[8], arcs[9], arcs[10], 0,
            ],
            12 => [
                arcs[2], arcs[3], arcs[4], arcs[5], arcs[6],
                arcs[7], arcs[8], arcs[9], arcs[10], arcs[11],
            ],
            _ => [0u32; MAX_LOWER_ARCS], // Checks above prevent this case, but makes Miri happy
        };

        // TODO(tarcieri): use `LowerArcs::new` when `const fn`-friendly
        Self {
            root_arcs,
            lower_arcs: LowerArcs {
                length: (arcs.len() - 2) as u8,
                arcs: lower_arcs,
            },
        }
    }

    /// Return the arc with the given index, if it exists.
    pub fn arc(&self, index: usize) -> Option<u32> {
        match index {
            0 => Some(self.root_arcs.first_arc()),
            1 => Some(self.root_arcs.second_arc()),
            n => self.lower_arcs.as_ref().get(n - 2).cloned(),
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
        2 + self.lower_arcs.len()
    }

    /// Parse an OID from from its BER/DER encoding.
    pub fn from_ber(mut bytes: &[u8]) -> Result<Self> {
        let root_arcs = parse_byte(&mut bytes).and_then(RootArcs::try_from)?;
        let lower_arcs = LowerArcs::from_ber(bytes)?;

        Ok(Self {
            root_arcs,
            lower_arcs,
        })
    }

    /// Get the length of this OID when serialized as ASN.1 BER.
    pub fn ber_len(&self) -> usize {
        // 1-byte from serialized `RootArcs`
        1 + self.lower_arcs.ber_len()
    }

    /// Write the BER encoding of this OID into the given slice, returning
    /// a new slice containing the written data.
    pub fn write_ber<'a>(&self, bytes: &'a mut [u8]) -> Result<&'a [u8]> {
        if bytes.is_empty() {
            return Err(Error);
        }

        bytes[0] = self.root_arcs.into();

        let mut offset = 1;

        for &arc in self.lower_arcs.as_ref() {
            offset += write_base128(&mut bytes[offset..], arc)?;
        }

        Ok(&bytes[..offset])
    }

    /// Serialize this OID as ASN.1 BER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn to_ber(&self) -> alloc::vec::Vec<u8> {
        let mut output = vec![0u8; self.ber_len()];
        self.write_ber(&mut output).expect("bad buffer size");
        output
    }
}

impl FromStr for ObjectIdentifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut split = s.split('.');
        let first_arc = split.next().and_then(|s| s.parse().ok()).ok_or(Error)?;
        let second_arc = split.next().and_then(|s| s.parse().ok()).ok_or(Error)?;
        let root_arcs = RootArcs::new(first_arc, second_arc)?;
        let lower_arcs = LowerArcs::from_split(&mut split)?;

        Ok(Self {
            root_arcs,
            lower_arcs,
        })
    }
}

impl TryFrom<&[u32]> for ObjectIdentifier {
    type Error = Error;

    fn try_from(arcs: &[u32]) -> Result<Self> {
        if arcs.len() < MIN_ARCS || arcs.len() > MAX_ARCS {
            return Err(Error);
        }

        let root_arcs = RootArcs::new(arcs[0], arcs[1])?;
        let lower_arcs = LowerArcs::try_from(&arcs[2..])?;

        Ok(Self {
            root_arcs,
            lower_arcs,
        })
    }
}

impl From<&ObjectIdentifier> for ObjectIdentifier {
    fn from(oid: &ObjectIdentifier) -> ObjectIdentifier {
        *oid
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
        let arc = self.oid.arc(self.index)?;
        self.index = self.index.checked_add(1).unwrap();
        Some(arc)
    }
}

/// Byte containing the first and second arcs of an OID.
///
/// This is represented this way in order to reduce the overall size of the
/// [`ObjectIdentifier`] struct.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct RootArcs(u8);

impl RootArcs {
    /// Create [`RootArcs`] from the first and second arc values represented
    /// as `u32` integers.
    fn new(first_arc: u32, second_arc: u32) -> Result<Self> {
        if first_arc > FIRST_ARC_MAX || second_arc > SECOND_ARC_MAX {
            return Err(Error);
        }

        let byte = (first_arc * (SECOND_ARC_MAX + 1)) as u8 | second_arc as u8;
        Ok(Self(byte))
    }

    /// Get the value of the first arc
    fn first_arc(self) -> u32 {
        self.0 as u32 / (SECOND_ARC_MAX + 1)
    }

    /// Get the value of the second arc
    fn second_arc(self) -> u32 {
        self.0 as u32 % (SECOND_ARC_MAX + 1)
    }
}

impl TryFrom<u8> for RootArcs {
    type Error = Error;

    fn try_from(octet: u8) -> Result<Self> {
        let first = octet as u32 / (SECOND_ARC_MAX + 1);
        let second = octet as u32 % (SECOND_ARC_MAX + 1);
        Self::new(first, second)
    }
}

impl From<RootArcs> for u8 {
    fn from(root_arcs: RootArcs) -> u8 {
        root_arcs.0
    }
}

/// "Lower" arcs beyond the first and second arcs in an OID.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct LowerArcs {
    /// Number of additional "lower" arcs.
    length: u8,

    /// "Lower" arc values.
    arcs: [u32; MAX_LOWER_ARCS],
}

impl LowerArcs {
    /// Create new [`LowerArcs`] from an array and length, validating length
    /// is in range (1..MAX_LOWER_ARCS)
    fn new(arcs: [u32; MAX_LOWER_ARCS], length: usize) -> Result<Self> {
        if length > 0 && length < MAX_LOWER_ARCS {
            Ok(Self {
                arcs,
                length: length as u8,
            })
        } else {
            Err(Error)
        }
    }

    /// Parse [`LowerArcs`] from ASN.1 BER.
    fn from_ber(mut bytes: &[u8]) -> Result<Self> {
        let mut arcs = [0u32; MAX_LOWER_ARCS];
        let mut index = 0;

        while !bytes.is_empty() {
            let byte = arcs.get_mut(index).ok_or(Error)?;
            *byte = parse_base128(&mut bytes)?;
            index = index.checked_add(1).unwrap();
        }

        Self::new(arcs, index)
    }

    /// Helper for parsing [`LowerArcs`] from a string
    fn from_split(split: &mut Split<'_, char>) -> Result<Self> {
        let mut arcs = [0u32; MAX_LOWER_ARCS];
        let mut length = 0;

        for (i, n) in split.enumerate() {
            let arc = arcs.get_mut(i).ok_or(Error)?;
            *arc = n.parse().map_err(|_| Error)?;
            length += 1;
        }

        Self::new(arcs, length)
    }

    /// Get the number of lower arcs
    pub fn len(&self) -> usize {
        self.length as usize
    }

    /// Get the length of the lower arcs when serialized as ASN.1 BER.
    pub fn ber_len(&self) -> usize {
        self.as_ref().iter().fold(0, |sum, n| sum + base128_len(*n))
    }
}

impl AsRef<[u32]> for LowerArcs {
    fn as_ref(&self) -> &[u32] {
        &self.arcs[..self.len()]
    }
}

impl TryFrom<&[u32]> for LowerArcs {
    type Error = Error;

    fn try_from(arcs: &[u32]) -> Result<Self> {
        if arcs.len() > MAX_LOWER_ARCS {
            return Err(Error);
        }

        let mut lower_arcs = [0u32; MAX_LOWER_ARCS];
        lower_arcs[..arcs.len()].copy_from_slice(arcs);

        Ok(Self {
            arcs: lower_arcs,
            length: arcs.len() as u8,
        })
    }
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
        0..=0x7f => 1,
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
        assert_eq!(EXAMPLE_OID.to_string(), EXAMPLE_OID_STRING);
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

        // Too short
        assert!("1.2".parse::<ObjectIdentifier>().is_err());

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

        // Too short
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
    fn new_too_short() {
        ObjectIdentifier::new(&[1, 2]);
    }

    #[test]
    #[should_panic]
    fn new_invalid_first_arc() {
        ObjectIdentifier::new(&[3, 2, 840, 10045, 3, 1, 7]);
    }

    #[test]
    #[should_panic]
    fn new_invalid_second_arc() {
        ObjectIdentifier::new(&[1, 40, 840, 10045, 3, 1, 7]);
    }
}
