//! Standard Base64 encoding *without* padding.
//!
//! ```text
//! [A-Z]      [a-z]      [0-9]      +     /
//! 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f
//! ```

use crate::{decoder, encoder, Error, InvalidEncodingError, InvalidLengthError, STD_HI_BYTES};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Decode a standard Base64 string without padding into the provided
/// destination buffer.
pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
    decoder::decode(src, dst, false, STD_HI_BYTES)
}

/// Decode a standard Base64 string without padding in-place.
pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
    decoder::decode_in_place(buf, false, STD_HI_BYTES)
}

/// Decode a standard Base64 string without padding into a byte vector.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
    decoder::decode_vec(input, false, STD_HI_BYTES)
}

/// Encode the input byte slice as standard Base64 with padding.
///
/// Writes the result into the provided destination slice, returning an
/// ASCII-encoded Base64 string value.
pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
    encoder::encode(src, dst, false, STD_HI_BYTES)
}

/// Encode input byte slice into a [`String`] containing standard Base64
/// without padding.
///
/// # Panics
/// If `input` length is greater than `usize::MAX/4`.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn encode_string(input: &[u8]) -> String {
    encoder::encode_string(input, false, STD_HI_BYTES)
}

/// Get the length of unpadded Base64 produced by encoding the given bytes.
///
/// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
pub fn encoded_len(bytes: &[u8]) -> usize {
    encoder::encoded_len(bytes, false)
}
