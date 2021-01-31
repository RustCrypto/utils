//! URL-safe Base64 encoding.
//!
//! ```text
//! [A-Z]      [a-z]      [0-9]      -     _
//! 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2d, 0x5f
//! ```

/// Encoding for bytes 62 and 63
const URL_HI_BYTES: (u8, u8) = (b'-', b'_');

/// URL-safe Base64 encoding with `=` padding.
pub mod padded {
    use super::URL_HI_BYTES;
    use crate::{decoder, encoder, Error, InvalidEncodingError, InvalidLengthError};

    #[cfg(feature = "alloc")]
    use alloc::{string::String, vec::Vec};

    /// Decode a URL-safe Base64 with padding string into the provided
    /// destination buffer.
    pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
        decoder::decode(src, dst, true, URL_HI_BYTES)
    }

    /// Decode a URL-safe Base64 string with padding in-place.
    pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
        decoder::decode_in_place(buf, true, URL_HI_BYTES)
    }

    /// Decode a URL-safe Base64 string with padding into a byte vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
        decoder::decode_vec(input, true, URL_HI_BYTES)
    }

    /// Encode the input byte slice as URL-safe Base64 with padding.
    ///
    /// Writes the result into the provided destination slice, returning an
    /// ASCII-encoded Base64 string value.
    pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
        encoder::encode(src, dst, true, URL_HI_BYTES)
    }

    /// Encode input byte slice into a [`String`] containing URL-safe Base64
    /// with padding.
    ///
    /// # Panics
    /// If `input` length is greater than `usize::MAX/4`.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn encode_string(input: &[u8]) -> String {
        encoder::encode_string(input, true, URL_HI_BYTES)
    }

    /// Get the length of padded Base64 produced by encoding the given bytes.
    ///
    /// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
    pub fn encoded_len(bytes: &[u8]) -> usize {
        encoder::encoded_len(bytes, true)
    }
}

/// URL-safe Base64 encoding *without* padding.
pub mod unpadded {
    use super::URL_HI_BYTES;
    use crate::{decoder, encoder, Error, InvalidEncodingError, InvalidLengthError};

    #[cfg(feature = "alloc")]
    use alloc::{string::String, vec::Vec};

    /// Decode a URL-safe Base64 string without padding into the provided
    /// destination buffer.
    pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
        decoder::decode(src, dst, false, URL_HI_BYTES)
    }

    /// Decode a URL-safe Base64 string without padding in-place.
    pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
        decoder::decode_in_place(buf, false, URL_HI_BYTES)
    }

    /// Decode a URL-safe Base64 string without padding into a byte vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
        decoder::decode_vec(input, false, URL_HI_BYTES)
    }

    /// Encode the input byte slice as URL-safe Base64 with padding.
    ///
    /// Writes the result into the provided destination slice, returning an
    /// ASCII-encoded Base64 string value.
    pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
        encoder::encode(src, dst, false, URL_HI_BYTES)
    }

    /// Encode input byte slice into a [`String`] containing URL-safe Base64
    /// without padding.
    ///
    /// # Panics
    /// If `input` length is greater than `usize::MAX/4`.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn encode_string(input: &[u8]) -> String {
        encoder::encode_string(input, false, URL_HI_BYTES)
    }

    /// Get the length of unpadded Base64 produced by encoding the given bytes.
    ///
    /// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
    pub fn encoded_len(bytes: &[u8]) -> usize {
        encoder::encoded_len(bytes, false)
    }
}
