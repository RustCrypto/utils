//! Arcs are integer values which exist within an OID's hierarchy.

use crate::{Error, ObjectIdentifier, Result};
use core::convert::TryFrom;

/// Type used to represent an "arc" (i.e. integer identifier value)
pub type Arc = u32;

/// Maximum value of the first arc in an OID
pub(crate) const FIRST_ARC_MAX: Arc = 2;

/// Maximum value of the second arc in an OID
pub(crate) const SECOND_ARC_MAX: Arc = 39;

/// [`Iterator`] over arcs (a.k.a. nodes) in an [`ObjectIdentifier`].
///
/// This iterates over all arcs in an OID, including the root.
pub struct Arcs<'a> {
    /// OID we're iterating over
    oid: &'a ObjectIdentifier,

    /// Current position within the serialized DER bytes of this OID
    cursor: Option<usize>,
}

impl<'a> Arcs<'a> {
    /// Create a new iterator over the arcs of this OID
    pub(crate) fn new(oid: &'a ObjectIdentifier) -> Self {
        Self { oid, cursor: None }
    }
}

impl<'a> Iterator for Arcs<'a> {
    type Item = Arc;

    fn next(&mut self) -> Option<Arc> {
        match self.cursor {
            // Indicates we're on the root OID
            None => {
                let root = RootArcs(self.oid.as_bytes()[0]);
                self.cursor = Some(0);
                Some(root.first_arc())
            }
            Some(0) => {
                let root = RootArcs(self.oid.as_bytes()[0]);
                self.cursor = Some(1);
                Some(root.second_arc())
            }
            Some(offset) => {
                let mut result = 0;
                let mut arc_bytes = 0;

                // TODO(tarcieri): consolidate this with `ObjectIdentifier::from_bytes`?
                loop {
                    match self.oid.as_bytes().get(offset + arc_bytes).cloned() {
                        Some(byte) => {
                            arc_bytes += 1;
                            assert!(
                                arc_bytes < 4 || byte & 0b11110000 == 0,
                                "OID arc overflowed"
                            );
                            result = result << 7 | (byte & 0b1111111) as Arc;

                            if byte & 0b10000000 == 0 {
                                self.cursor = Some(offset + arc_bytes);
                                return Some(result);
                            }
                        }
                        None => {
                            assert_eq!(arc_bytes, 0, "truncated OID");
                            return None;
                        }
                    }
                }
            }
        }
    }
}

/// Byte containing the first and second arcs of an OID.
///
/// This is represented this way in order to reduce the overall size of the
/// [`ObjectIdentifier`] struct.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootArcs(u8);

impl RootArcs {
    /// Create [`RootArcs`] from the first and second arc values represented
    /// as `Arc` integers.
    pub(crate) fn new(first_arc: Arc, second_arc: Arc) -> Result<Self> {
        if first_arc > FIRST_ARC_MAX || second_arc > SECOND_ARC_MAX {
            return Err(Error);
        }

        let byte = (first_arc * (SECOND_ARC_MAX + 1)) as u8 + second_arc as u8;
        Ok(Self(byte))
    }

    /// Get the value of the first arc
    pub(crate) fn first_arc(self) -> Arc {
        self.0 as Arc / (SECOND_ARC_MAX + 1)
    }

    /// Get the value of the second arc
    pub(crate) fn second_arc(self) -> Arc {
        self.0 as Arc % (SECOND_ARC_MAX + 1)
    }
}

impl TryFrom<u8> for RootArcs {
    type Error = Error;

    fn try_from(octet: u8) -> Result<Self> {
        let first = octet as Arc / (SECOND_ARC_MAX + 1);
        let second = octet as Arc % (SECOND_ARC_MAX + 1);
        let result = Self::new(first, second)?;
        debug_assert_eq!(octet, result.0);
        Ok(result)
    }
}

impl From<RootArcs> for u8 {
    fn from(root_arcs: RootArcs) -> u8 {
        root_arcs.0
    }
}
