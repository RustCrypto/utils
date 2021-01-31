//! bcrypt Base64 encoding
//!
//! ```text
//! ./         [A-Z]      [a-z]     [0-9]
//! 0x2e-0x2f, 0x41-0x5a, 0x61-0x7a, 0x30-0x39
//! ```

use crate::{
    decoder::{self, match_range_ct},
    encoder::{self, match_gt_ct},
    Error, InvalidEncodingError, InvalidLengthError,
};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Decode a bcrypt Base64 string into the provided
/// destination buffer.
pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
    decoder::decode(src, dst, false, decode_6bits)
}

/// Decode a bcrypt Base64 string in-place.
pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
    decoder::decode_in_place(buf, false, decode_6bits)
}

/// Decode a bcrypt Base64 string into a byte vector.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
    decoder::decode_vec(input, false, decode_6bits)
}

/// Encode the input byte slice as bcrypt Base64 with padding.
///
/// Writes the result into the provided destination slice, returning an
/// ASCII-encoded Base64 string value.
pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
    encoder::encode(src, dst, false, encode_6bits)
}

/// Encode input byte slice into a [`String`] containing bcrypt Base64
/// without padding.
///
/// # Panics
/// If `input` length is greater than `usize::MAX/4`.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn encode_string(input: &[u8]) -> String {
    encoder::encode_string(input, false, encode_6bits)
}

/// Get the length of Base64 produced by encoding the given bytes.
///
/// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
pub fn encoded_len(bytes: &[u8]) -> usize {
    encoder::encoded_len(bytes, false)
}

#[inline(always)]
fn decode_6bits(src: u8) -> i16 {
    let mut res: i16 = -1;
    res += match_range_ct(src, b'.'..b'/', src as i16 - 45);
    res += match_range_ct(src, b'A'..b'Z', src as i16 - 62);
    res += match_range_ct(src, b'a'..b'z', src as i16 - 68);
    res + match_range_ct(src, b'0'..b'9', src as i16 + 7)
}

#[inline(always)]
fn encode_6bits(mut src: i16) -> u8 {
    src += 0x2e;
    src += match_gt_ct(src, 0x2f, 17);
    src += match_gt_ct(src, 0x5a, 6);
    src -= match_gt_ct(src, 0x7a, 75);
    src as u8
}
