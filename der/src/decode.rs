//! ASN.1 DER decoding support.

use crate::{Error, Result, Tag};

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// Decode `INTEGER`.
pub fn integer(bytes: &mut &[u8]) -> Result<usize> {
    if byte(bytes)? != Tag::Integer as u8 {
        return Err(Error);
    }

    // We presently specialize for 1-byte integers to parse versions
    if length(bytes)? == 1 {
        Ok(byte(bytes)? as usize)
    } else {
        Err(Error)
    }
}

/// Decode [`ObjectIdentifier`].
#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub fn oid(bytes: &mut &[u8]) -> Result<ObjectIdentifier> {
    Ok(ObjectIdentifier::from_ber(bytes)?)
}

/// Decode nested value (e.g. `OCTET STRING`, `SEQUENCE`).
pub fn nested<'a>(bytes: &mut &'a [u8], expected_tag: Tag) -> Result<&'a [u8]> {
    if byte(bytes)? != expected_tag as u8 {
        return Err(Error);
    }

    let len = length(bytes)?;

    if len <= bytes.len() {
        let (head, tail) = bytes.split_at(len);
        *bytes = tail;
        Ok(head)
    } else {
        Err(Error)
    }
}

/// Extract the leading byte from a slice.
///
/// This function provides a panic-free foundation for parsing single bytes.
// TODO(tarcieri): encapsulate and remove from public API
pub fn byte(bytes: &mut &[u8]) -> Result<u8> {
    let byte = *bytes.get(0).ok_or(Error)?;
    *bytes = &bytes[1..];
    Ok(byte)
}

/// Parse DER-encoded length.
///
/// This function supports lengths up to 65,535 bytes.
// TODO(tarcieri): encapsulate and remove from public API
pub fn length(bytes: &mut &[u8]) -> Result<usize> {
    match byte(bytes)? {
        // Note: per X.690 Section 8.1.3.6.1 the byte 0x80 encodes indefinite
        // lengths, which are not allowed in DER
        len if len < 0x80 => Ok(len as usize),
        0x81 => {
            let len = byte(bytes)? as usize;

            // X.690 Section 10.1: DER lengths must be encoded with a minimum
            // number of octets
            if len >= 0x80 {
                Ok(len)
            } else {
                Err(Error)
            }
        }
        0x82 => {
            let len_hi = byte(bytes)? as usize;
            let len = (len_hi << 8) | (byte(bytes)? as usize);

            // X.690 Section 10.1: DER lengths must be encoded with a minimum
            // number of octets
            if len > 0xFF {
                Ok(len)
            } else {
                Err(Error)
            }
        }
        _ => {
            // We specialize to a maximum 3-byte length
            Err(Error)
        }
    }
}
