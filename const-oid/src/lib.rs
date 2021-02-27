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
//! The BER/DER encoding of OIDs supported by this library MUST be shorter than
//! the [`MAX_LEN`] constant.
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
    html_root_url = "https://docs.rs/const-oid/0.4.3"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod macros;

mod arcs;
mod encoder;
mod error;
mod parser;

pub use crate::{
    arcs::{Arc, Arcs},
    error::{Error, Result},
};

use crate::arcs::RootArcs;
use core::{convert::TryFrom, fmt, str::FromStr};

/// Minimum number of arcs in an OID.
///
/// This library will refuse to parse OIDs with fewer than this number of arcs.
pub const MIN_ARCS: usize = 3;

/// Maximum number of arcs in an OID.
#[deprecated(since = "0.4.4", note = "Please use the `MAX_LEN` instead")]
pub const MAX_ARCS: usize = 12;

/// Maximum length of a DER-encoded OID in bytes.
pub const MAX_LEN: usize = 23;

/// Object identifier (OID).
///
/// OIDs are hierarchical structures consisting of "arcs", i.e. integer
/// identifiers.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ObjectIdentifier {
    /// Array containing BER/DER-serialized bytes (no header)
    bytes: [u8; MAX_LEN],

    /// Length in bytes
    length: u8,
}

#[allow(clippy::len_without_is_empty)]
impl ObjectIdentifier {
    /// Create an [`ObjectIdentifier`] from a slice of integers, where each
    /// integer represents an "arc" (a.k.a. node) in the OID.
    ///
    /// NOTE: this method is soft-deprecated and will be removed in a future
    /// release. We recommend using [`ObjectIdentifier::parse`] going forward
    /// (which will be renamed to [`ObjectIdentifier::new`] in a future release).
    ///
    /// # Panics
    ///
    /// To enable `const fn` usage and work around current limitations thereof,
    /// this method panics in the event the OID is malformed.
    ///
    /// For that reason this method is not recommended except for use in
    /// constants (where it will generate a compiler error instead).
    /// To parse an OID from a `&[Arc]` slice without panicking on error,
    /// use [`TryFrom<&[Arc]>`][1] instead.
    ///
    /// In order for an OID to be valid, it must meet the following criteria:
    ///
    /// - The OID MUST have at least 3 arcs
    /// - The first arc MUST be within the range 0-2
    /// - The second arc MUST be within the range 0-39
    /// - The BER/DER encoding of the OID MUST be shorter than the [`MAX_LEN`] constant
    ///
    /// [1]: ./struct.ObjectIdentifier.html#impl-TryFrom%3C%26%27_%20%5BArc%5D%3E
    pub const fn new(arcs: &[Arc]) -> Self {
        const_assert!(arcs.len() >= MIN_ARCS, "OID too short (minimum 3 arcs)");
        let mut encoder = encoder::Encoder::new();

        macro_rules! encode_arc {
            ($($n:expr),+) => {
                $(
                    if arcs.len() > $n {
                        encoder = encoder.encode(arcs[$n]);
                    }
                )+
             };
        }

        encode_arc!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
        encoder.finish()
    }

    /// Parse an [`ObjectIdentifier`] from the dot-delimited string form, e.g.:
    ///
    /// ```
    /// use const_oid::ObjectIdentifier;
    ///
    /// const MY_OID: ObjectIdentifier = ObjectIdentifier::parse("1.2.840.113549.1.1.1");
    /// ```
    ///
    /// Like [`ObjectIdentifier::new`], this version is intended for use in
    /// `const` contexts. where it will generate compile errors in the event
    /// the OID is malformed.
    ///
    /// This method is *NOT* intended for use outside of const contexts, as it
    /// will panic with a bad error message. However, this type also has a
    /// [`FromStr`] impl that can be used for fallible parsing.
    pub const fn parse(s: &str) -> Self {
        parser::Parser::parse(s).finish()
    }

    /// Get the BER/DER serialization of this OID as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length as usize]
    }

    /// Return the arc with the given index, if it exists.
    pub fn arc(&self, index: usize) -> Option<Arc> {
        self.arcs().nth(index)
    }

    /// Iterate over the arcs (a.k.a. nodes) in an [`ObjectIdentifier`].
    ///
    /// Returns [`Arcs`], an iterator over `Arc` values representing the value
    /// of each arc/node.
    pub fn arcs(&self) -> Arcs<'_> {
        Arcs::new(self)
    }

    /// Number of arcs in this [`ObjectIdentifier`].
    pub fn len(&self) -> usize {
        self.arcs().count()
    }

    /// Parse an OID from from its BER/DER encoding.
    pub fn from_ber(ber_bytes: &[u8]) -> Result<Self> {
        let len = ber_bytes.len();

        if !(2..=MAX_LEN).contains(&len) {
            return Err(Error);
        }

        // Validate root arcs are in range
        ber_bytes
            .get(0)
            .cloned()
            .ok_or(Error)
            .and_then(RootArcs::try_from)?;

        // Validate lower arcs are well-formed
        let mut arc_offset = 1;
        let mut arc_bytes = 0;

        // TODO(tarcieri): consolidate this with `Arcs::next`?
        while arc_offset < len {
            match ber_bytes.get(arc_offset + arc_bytes).cloned() {
                Some(byte) => {
                    arc_bytes += 1;

                    if arc_bytes == 4 && byte & 0b11110000 != 0 {
                        // Overflowed `Arc` (u32)
                        return Err(Error);
                    }

                    if byte & 0b10000000 == 0 {
                        arc_offset += arc_bytes;
                        arc_bytes = 0;
                    }
                }
                None => return Err(Error), // truncated OID
            }
        }

        let mut bytes = [0u8; MAX_LEN];
        bytes[..len].copy_from_slice(ber_bytes);

        Ok(Self {
            bytes,
            length: len as u8,
        })
    }

    /// Get the length of this OID when serialized as ASN.1 BER.
    #[deprecated(since = "0.4.4", note = "Please use the `as_bytes()` function instead")]
    pub fn ber_len(&self) -> usize {
        self.as_bytes().len()
    }

    /// Write the BER encoding of this OID into the given slice, returning
    /// a new slice containing the written data.
    #[deprecated(since = "0.4.4", note = "Please use the `as_bytes()` function instead")]
    pub fn write_ber<'a>(&self, bytes: &'a mut [u8]) -> Result<&'a [u8]> {
        let len = self.as_bytes().len();

        if bytes.len() < len {
            return Err(Error);
        }

        bytes[..len].copy_from_slice(self.as_bytes());
        Ok(&bytes[..len])
    }

    /// Serialize this OID as ASN.1 BER.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    #[deprecated(since = "0.4.4", note = "Please use the `as_bytes()` function instead")]
    pub fn to_ber(&self) -> alloc::vec::Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl AsRef<[u8]> for ObjectIdentifier {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl FromStr for ObjectIdentifier {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        let mut split = string.split('.');
        let first_arc = split.next().and_then(|s| s.parse().ok()).ok_or(Error)?;
        let second_arc = split.next().and_then(|s| s.parse().ok()).ok_or(Error)?;

        let mut bytes = [0u8; MAX_LEN];
        bytes[0] = RootArcs::new(first_arc, second_arc)?.into();

        let mut offset = 1;

        for s in split {
            let arc = s.parse().map_err(|_| Error)?;
            offset += encoder::write_base128(&mut bytes[offset..], arc)?;
        }

        if offset > 1 {
            Ok(Self {
                bytes,
                length: offset as u8,
            })
        } else {
            // Minimum 3 arcs
            Err(Error)
        }
    }
}

impl TryFrom<&[Arc]> for ObjectIdentifier {
    type Error = Error;

    fn try_from(arcs: &[Arc]) -> Result<Self> {
        if arcs.len() < MIN_ARCS {
            return Err(Error);
        }

        let mut bytes = [0u8; MAX_LEN];
        bytes[0] = RootArcs::new(arcs[0], arcs[1])?.into();

        let mut offset = 1;

        for &arc in &arcs[2..] {
            offset += encoder::write_base128(&mut bytes[offset..], arc)?;
        }

        Ok(Self {
            bytes,
            length: offset as u8,
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
        let len = self.len();

        for (i, arc) in self.arcs().enumerate() {
            write!(f, "{}", arc)?;

            if i < len - 1 {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}
