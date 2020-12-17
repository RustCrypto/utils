//! ASN.1 DER encoding support.

use crate::{Error, Result, Tag};

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// Encode a tag and a length header
pub fn header(buffer: &mut [u8], tag: Tag, len: usize) -> Result<usize> {
    byte(buffer, 0, tag as u8)?;
    length(&mut buffer[1..], len).and_then(|len| len.checked_add(1).ok_or(Error::Overflow))
}

/// Encode nested value (e.g. `OCTET STRING`, `SEQUENCE`).
pub fn nested(buffer: &mut [u8], tag: Tag, data: &[u8]) -> Result<usize> {
    let offset = header(buffer, tag, data.len())?;

    if buffer[offset..].len() < data.len() {
        return Err(Error::Overlength);
    }

    buffer[offset..(offset + data.len())].copy_from_slice(data);
    offset.checked_add(data.len()).ok_or(Error::Overflow)
}

/// Encode [`ObjectIdentifier`].
#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub fn oid(buffer: &mut [u8], oid: ObjectIdentifier) -> Result<usize> {
    let offset = header(buffer, Tag::ObjectIdentifier, oid.ber_len())?;

    offset
        .checked_add(oid.write_ber(&mut buffer[offset..])?.len())
        .ok_or(Error::Overflow)
}

/// Encode a single byte at the given offset
fn byte(buffer: &mut [u8], offset: usize, byte: u8) -> Result<()> {
    buffer
        .get_mut(offset)
        .map(|b| *b = byte)
        .ok_or(Error::Overlength)
}

/// Encode length prefix.
///
/// This function supports lengths up to 65,535 bytes.
fn length(buffer: &mut [u8], len: usize) -> Result<usize> {
    match len {
        0..=0x7F => {
            byte(buffer, 0, len as u8)?;
            Ok(1)
        }
        0x80..=0xFF => {
            byte(buffer, 0, 0x81)?;
            byte(buffer, 1, len as u8)?;
            Ok(2)
        }
        0x100..=0xFFFF => {
            byte(buffer, 0, 0x82)?;
            byte(buffer, 1, (len >> 8) as u8)?;
            byte(buffer, 2, (len & 0xFF) as u8)?;
            Ok(3)
        }
        _ => Err(Error::Overlength),
    }
}
