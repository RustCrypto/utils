//! Length calculations for encoded ASN.1 DER values

use crate::{Decodable, Decoder, Error, Result};
use core::convert::TryFrom;

/// ASN.1-encoded length.
///
/// # Limits
///
/// Presently constrained to the range `0..=65535`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Length(u16);

impl Length {
    /// Get the maximum length supported by this crate
    pub const fn max() -> usize {
        u16::MAX as usize
    }
}

impl From<u8> for Length {
    fn from(len: u8) -> Length {
        Length(len as u16)
    }
}

impl From<u16> for Length {
    fn from(len: u16) -> Length {
        Length(len)
    }
}

impl From<Length> for u16 {
    fn from(len: Length) -> u16 {
        len.0
    }
}

impl From<Length> for usize {
    fn from(len: Length) -> usize {
        len.0 as usize
    }
}

impl TryFrom<usize> for Length {
    type Error = Error;

    fn try_from(len: usize) -> Result<Length> {
        u16::try_from(len).map(Length).map_err(|_| Error::Overflow)
    }
}

impl Decodable<'_> for Length {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Length> {
        // TODO(tarcieri): move `decode::byte` to `Decoder`
        match crate::decoder::byte(decoder)? {
            // Note: per X.690 Section 8.1.3.6.1 the byte 0x80 encodes indefinite
            // lengths, which are not allowed in DER, so disallow that byte.
            len if len < 0x80 => Ok(len.into()),
            0x81 => {
                let len = crate::decoder::byte(decoder)?;

                // X.690 Section 10.1: DER lengths must be encoded with a minimum
                // number of octets
                if len >= 0x80 {
                    Ok(len.into())
                } else {
                    Err(Error::Noncanonical)
                }
            }
            0x82 => {
                let len_hi = crate::decoder::byte(decoder)? as u16;
                let len = (len_hi << 8) | (crate::decoder::byte(decoder)? as u16);

                // X.690 Section 10.1: DER lengths must be encoded with a minimum
                // number of octets
                if len > 0xFF {
                    Ok(len.into())
                } else {
                    Err(Error::Noncanonical)
                }
            }
            _ => {
                // We specialize to a maximum 3-byte length
                Err(Error::Overlength)
            }
        }
    }
}

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

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
