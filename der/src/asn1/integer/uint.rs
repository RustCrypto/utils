//! Unsigned integer decoders/encoders.

use crate::{asn1::Any, Encodable, Encoder, Header, Length, Result, Tag};
use core::convert::TryFrom;

/// Decode an unsigned integer into a big endian byte slice with all leading
/// zeroes removed.
///
/// Returns a byte array of the requested size containing a big endian integer.
pub(super) fn decode_slice(any: Any<'_>) -> Result<&[u8]> {
    let tag = any.tag().assert_eq(Tag::Integer)?;
    let bytes = any.as_bytes();

    // The `INTEGER` type always encodes a signed value, so for unsigned
    // values the leading `0x00` byte may need to be removed.
    //
    // We also disallow a leading byte which would overflow a signed ASN.1
    // integer (since we're decoding an unsigned integer).
    // We expect all such cases to have a leading `0x00` byte.
    match bytes {
        [] => Err(tag.non_canonical_error()),
        [0] => Ok(bytes),
        [0, byte, ..] if *byte < 0x80 => Err(tag.non_canonical_error()),
        [0, rest @ ..] => Ok(&rest),
        [byte, ..] if *byte >= 0x80 => Err(tag.value_error()),
        _ => Ok(bytes),
    }
}

/// Decode an unsigned integer into a byte array of the requested size
/// containing a big endian integer.
pub(super) fn decode_array<const N: usize>(any: Any<'_>) -> Result<[u8; N]> {
    let input = decode_slice(any)?;

    // Input has leading zeroes removed, so we need to add them back
    let mut output = [0u8; N];
    output[N.saturating_sub(input.len())..].copy_from_slice(input);
    Ok(output)
}

/// Encode the given big endian bytes representing an integer as ASN.1 DER.
pub(super) fn encode(encoder: &mut Encoder<'_>, bytes: &[u8]) -> Result<()> {
    let bytes = strip_leading_zeroes(&bytes);
    let leading_zero = needs_leading_zero(bytes);
    let len = (Length::try_from(bytes.len())? + leading_zero as u8)?;
    Header::new(Tag::Integer, len)?.encode(encoder)?;

    if leading_zero {
        encoder.byte(0)?;
    }

    encoder.bytes(bytes)
}

/// Get the encoded length for the given unsigned integer serialized as bytes.
#[inline]
pub(super) fn encoded_len(bytes: &[u8]) -> Result<Length> {
    let bytes = strip_leading_zeroes(&bytes);
    Length::try_from(bytes.len())? + needs_leading_zero(bytes) as u8
}

/// Strip the leading zeroes from the given byte slice
pub(super) fn strip_leading_zeroes(mut bytes: &[u8]) -> &[u8] {
    while let Some((byte, rest)) = bytes.split_first() {
        if *byte == 0 && !rest.is_empty() {
            bytes = rest;
        } else {
            break;
        }
    }

    bytes
}

/// Does the given integer need a leading zero?
fn needs_leading_zero(bytes: &[u8]) -> bool {
    matches!(bytes.get(0), Some(byte) if *byte >= 0x80)
}
