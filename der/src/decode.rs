//! ASN.1 DER decoding support.

use crate::{Error, Result, Tag};
use core::convert::TryFrom;

#[cfg(feature = "oid")]
use crate::ObjectIdentifier;

/// Extract the leading byte from a slice.
///
/// This function provides a panic-free foundation for parsing single bytes.
// TODO(tarcieri): encapsulate and remove from public API
pub fn byte(bytes: &mut &[u8]) -> Result<u8> {
    let byte = *bytes.get(0).ok_or(Error::Truncated)?;
    *bytes = &bytes[1..];
    Ok(byte)
}

/// Decode a [`Tag`] value
pub fn tag(bytes: &mut &[u8]) -> Result<Tag> {
    byte(bytes).and_then(Tag::try_from)
}

/// Decode `INTEGER`.
pub fn integer(bytes: &mut &[u8]) -> Result<usize> {
    let tag = tag(bytes)?.expect(Tag::Integer)?;

    // TODO(tarcieri): support for INTEGER values longer than 1-byte
    if length(bytes)? == 1 {
        Ok(byte(bytes)? as usize)
    } else {
        Err(Error::Length { tag })
    }
}

/// Decode `ANY` TLV-encoded ASN.1 value, calling the provided [`FnOnce`] with
/// the [`Tag`] and the value upon success, and returning the result.
pub fn any<'a, F, T>(bytes: &mut &'a [u8], f: F) -> Result<T>
where
    F: FnOnce(Tag, &'a [u8]) -> Result<T>,
{
    let tag = tag(bytes)?;
    let len = length(bytes)?;

    if len > bytes.len() {
        return Err(Error::Length { tag });
    }

    let (head, tail) = bytes.split_at(len);
    *bytes = tail;
    f(tag, head)
}

/// Parse an `OPTIONAL` value, calling the provided [`FnOnce`] if the value is
/// present and returning `Some` on success, or `None` if the value is absent.
pub fn optional<'a, F, T>(bytes: &mut &'a [u8], f: F) -> Result<Option<T>>
where
    F: FnOnce(Tag, &'a [u8]) -> Result<T>,
{
    if bytes.is_empty() {
        Ok(None)
    } else {
        any(bytes, f).map(Some)
    }
}

/// Expect a TLV-encoded value with the given [`Tag`], calling the provided
/// [`FnOnce`] with the value if the tag matches.
pub fn tagged<'a, F, T>(bytes: &mut &'a [u8], expected_tag: Tag, f: F) -> Result<T>
where
    F: FnOnce(&'a [u8]) -> Result<T>,
{
    any(bytes, |tag, inner| {
        tag.expect(expected_tag)?;
        f(inner)
    })
}

/// Decode [`ObjectIdentifier`].
#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub fn oid(bytes: &mut &[u8]) -> Result<ObjectIdentifier> {
    tagged(bytes, Tag::ObjectIdentifier, |oid| {
        Ok(ObjectIdentifier::from_ber(oid)?)
    })
}

/// Decode `BIT STRING`
pub fn bit_string<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8]> {
    tagged(bytes, Tag::BitString, Ok)
}

/// Decode `OCTET STRING`
pub fn octet_string<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8]> {
    tagged(bytes, Tag::OctetString, Ok)
}

/// Decode `SEQUENCE`.
pub fn sequence<'a, F, T>(bytes: &mut &'a [u8], f: F) -> Result<T>
where
    F: FnOnce(&'a [u8]) -> Result<T>,
{
    tagged(bytes, Tag::Sequence, f)
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
                Err(Error::Noncanonical)
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
                Err(Error::Noncanonical)
            }
        }
        _ => {
            // We specialize to a maximum 3-byte length
            Err(Error::Overlength)
        }
    }
}
