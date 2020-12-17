//! Length calculations for encoded ASN.1 DER values

use crate::{Error, Result};

#[cfg(feature = "oid")]
use oid::ObjectIdentifier;

/// Compute the length of a header including the tag byte.
///
/// This function supports `nested_len` values up to 65,535 bytes.
pub fn header(nested_len: usize) -> Result<usize> {
    match nested_len {
        0..=0x7F => Ok(2),
        0x80..=0xFF => Ok(3),
        0x100..=0xFFFF => Ok(4),
        _ => Err(Error::Overlength),
    }
}

/// Get the length of a DER-encoded OID
#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub fn oid(oid: ObjectIdentifier) -> Result<usize> {
    let body_len = oid.ber_len();
    header(body_len)?
        .checked_add(body_len)
        .ok_or(Error::Overflow)
}
