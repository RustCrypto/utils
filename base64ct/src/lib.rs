//! Pure Rust implementation of Base64 encoding ([RFC 4648, section 4])
//! with a constant-time `no_std`-friendly implementation.
//!
//! # About
//!
//! This crate implements the following Base64 variants in constant-time:
//!
//! - Standard Base64 encoding: `[A-Za-z0-9+/]`
//!   - [`base64ct::padded`][`padded`]
//!   - [`base64ct::unpadded`][`unpadded`]
//! - URL-safe Base64: `[A-Za-z0-9\-_]`
//!   - [`base64ct::url::padded`][`url::padded`]
//!   - [`base64ct::url::unpadded`][`url::unpadded`]
//!
//! The padded variants require (`=`) padding. Unpadded variants expressly
//! reject such padding.
//!
//! Whitespace is expressly disallowed.
//!
//! # Usage
//!
//! ## Allocating (enable `alloc` crate feature)
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use base64ct::padded as base64;
//!
//! let bytes = b"example bytestring!";
//! let encoded = base64::encode_string(bytes);
//! assert_eq!(encoded, "ZXhhbXBsZSBieXRlc3RyaW5nIQ==");
//!
//! let decoded = base64::decode_vec(&encoded).unwrap();
//! assert_eq!(decoded, bytes);
//! # }
//! ```
//!
//! ## Heapless `no_std` usage
//!
//! ```
//! use base64ct::padded as base64;
//!
//! const BUF_SIZE: usize = 128;
//!
//! let bytes = b"example bytestring!";
//! assert!(base64::encoded_len(bytes) <= BUF_SIZE);
//!
//! let mut enc_buf = [0u8; BUF_SIZE];
//! let encoded = base64::encode(bytes, &mut enc_buf).unwrap();
//! assert_eq!(encoded, "ZXhhbXBsZSBieXRlc3RyaW5nIQ==");
//!
//! let mut dec_buf = [0u8; BUF_SIZE];
//! let decoded = base64::decode(encoded, &mut dec_buf).unwrap();
//! assert_eq!(decoded, bytes);
//! ```
//!
//! # Implementation
//!
//! Implemented using bitwise arithmetic alone without any lookup tables or
//! data-dependent branches, thereby providing portable "best effort"
//! constant-time operation.
//!
//! Not constant-time with respect to message length (only data).
//!
//! Adapted from the following constant-time C++ implementation of Base64:
//!
//! <https://github.com/Sc00bz/ConstTimeEncoding/blob/master/base64.cpp>
//!
//! Copyright (c) 2014 Steve "Sc00bz" Thomas (steve at tobtu dot com).
//! Derived code is dual licensed MIT + Apache 2 (with permission from Sc00bz).
//!
//! [RFC 4648, section 4]: https://tools.ietf.org/html/rfc4648#section-4

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/base64ct/0.1.1"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod errors;

pub use errors::{Error, InvalidEncodingError, InvalidLengthError};

use core::{ops::Range, str};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Padding character
const PAD: u8 = b'=';

/// Standard encoding for bytes 62 and 63
const STD_HI_BYTES: (u8, u8) = (b'+', b'/');

/// Standard Base64 encoding with `=` padding.
///
/// ```text
/// [A-Z]      [a-z]      [0-9]      +     /
/// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f
/// ```
pub mod padded {
    use crate::{Error, InvalidEncodingError, InvalidLengthError, STD_HI_BYTES};

    #[cfg(feature = "alloc")]
    use alloc::{string::String, vec::Vec};

    /// Decode a standard Base64 with padding string into the provided
    /// destination buffer.
    pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
        crate::decode(src, dst, true, STD_HI_BYTES)
    }

    /// Decode a standard Base64 string with padding in-place.
    pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
        crate::decode_in_place(buf, true, STD_HI_BYTES)
    }

    /// Decode a standard Base64 string with padding into a byte vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
        crate::decode_vec(input, true, STD_HI_BYTES)
    }

    /// Encode the input byte slice as standard Base64 with padding.
    ///
    /// Writes the result into the provided destination slice, returning an
    /// ASCII-encoded Base64 string value.
    pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
        crate::encode(src, dst, true, STD_HI_BYTES)
    }

    /// Encode input byte slice into a [`String`] containing standard Base64
    /// with padding.
    ///
    /// # Panics
    /// If `input` length is greater than `usize::MAX/4`.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn encode_string(input: &[u8]) -> String {
        crate::encode_string(input, true, STD_HI_BYTES)
    }

    /// Get the length of padded Base64 produced by encoding the given bytes.
    ///
    /// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
    pub fn encoded_len(bytes: &[u8]) -> usize {
        crate::encoded_len(bytes, true)
    }
}

/// Standard Base64 encoding *without* padding.
///
/// ```text
/// [A-Z]      [a-z]      [0-9]      +     /
/// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f
/// ```
pub mod unpadded {
    use crate::{Error, InvalidEncodingError, InvalidLengthError, STD_HI_BYTES};

    #[cfg(feature = "alloc")]
    use alloc::{string::String, vec::Vec};

    /// Decode a standard Base64 string without padding into the provided
    /// destination buffer.
    pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
        crate::decode(src, dst, false, STD_HI_BYTES)
    }

    /// Decode a standard Base64 string without padding in-place.
    pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
        crate::decode_in_place(buf, false, STD_HI_BYTES)
    }

    /// Decode a standard Base64 string without padding into a byte vector.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
        crate::decode_vec(input, false, STD_HI_BYTES)
    }

    /// Encode the input byte slice as standard Base64 with padding.
    ///
    /// Writes the result into the provided destination slice, returning an
    /// ASCII-encoded Base64 string value.
    pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
        crate::encode(src, dst, false, STD_HI_BYTES)
    }

    /// Encode input byte slice into a [`String`] containing standard Base64
    /// without padding.
    ///
    /// # Panics
    /// If `input` length is greater than `usize::MAX/4`.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    pub fn encode_string(input: &[u8]) -> String {
        crate::encode_string(input, false, STD_HI_BYTES)
    }

    /// Get the length of unpadded Base64 produced by encoding the given bytes.
    ///
    /// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
    pub fn encoded_len(bytes: &[u8]) -> usize {
        crate::encoded_len(bytes, false)
    }
}

/// URL-safe Base64 encoding.
///
/// ```text
/// [A-Z]      [a-z]      [0-9]      -     _
/// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2d, 0x5f
/// ```
pub mod url {
    /// Encoding for bytes 62 and 63
    const URL_HI_BYTES: (u8, u8) = (b'-', b'_');

    /// URL-safe Base64 encoding with `=` padding.
    pub mod padded {
        use super::URL_HI_BYTES;
        use crate::{Error, InvalidEncodingError, InvalidLengthError};

        #[cfg(feature = "alloc")]
        use alloc::{string::String, vec::Vec};

        /// Decode a URL-safe Base64 with padding string into the provided
        /// destination buffer.
        pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
            crate::decode(src, dst, true, URL_HI_BYTES)
        }

        /// Decode a URL-safe Base64 string with padding in-place.
        pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
            crate::decode_in_place(buf, true, URL_HI_BYTES)
        }

        /// Decode a URL-safe Base64 string with padding into a byte vector.
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
            crate::decode_vec(input, true, URL_HI_BYTES)
        }

        /// Encode the input byte slice as URL-safe Base64 with padding.
        ///
        /// Writes the result into the provided destination slice, returning an
        /// ASCII-encoded Base64 string value.
        pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
            crate::encode(src, dst, true, URL_HI_BYTES)
        }

        /// Encode input byte slice into a [`String`] containing URL-safe Base64
        /// with padding.
        ///
        /// # Panics
        /// If `input` length is greater than `usize::MAX/4`.
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub fn encode_string(input: &[u8]) -> String {
            crate::encode_string(input, true, URL_HI_BYTES)
        }

        /// Get the length of padded Base64 produced by encoding the given bytes.
        ///
        /// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
        pub fn encoded_len(bytes: &[u8]) -> usize {
            crate::encoded_len(bytes, true)
        }
    }

    /// URL-safe Base64 encoding *without* padding.
    pub mod unpadded {
        use super::URL_HI_BYTES;
        use crate::{Error, InvalidEncodingError, InvalidLengthError};

        #[cfg(feature = "alloc")]
        use alloc::{string::String, vec::Vec};

        /// Decode a URL-safe Base64 string without padding into the provided
        /// destination buffer.
        pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8]) -> Result<&[u8], Error> {
            crate::decode(src, dst, false, URL_HI_BYTES)
        }

        /// Decode a URL-safe Base64 string without padding in-place.
        pub fn decode_in_place(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
            crate::decode_in_place(buf, false, URL_HI_BYTES)
        }

        /// Decode a URL-safe Base64 string without padding into a byte vector.
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub fn decode_vec(input: &str) -> Result<Vec<u8>, Error> {
            crate::decode_vec(input, false, URL_HI_BYTES)
        }

        /// Encode the input byte slice as URL-safe Base64 with padding.
        ///
        /// Writes the result into the provided destination slice, returning an
        /// ASCII-encoded Base64 string value.
        pub fn encode<'a>(src: &[u8], dst: &'a mut [u8]) -> Result<&'a str, InvalidLengthError> {
            crate::encode(src, dst, false, URL_HI_BYTES)
        }

        /// Encode input byte slice into a [`String`] containing URL-safe Base64
        /// without padding.
        ///
        /// # Panics
        /// If `input` length is greater than `usize::MAX/4`.
        #[cfg(feature = "alloc")]
        #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
        pub fn encode_string(input: &[u8]) -> String {
            crate::encode_string(input, false, URL_HI_BYTES)
        }

        /// Get the length of unpadded Base64 produced by encoding the given bytes.
        ///
        /// WARNING: this function will return `0` for lengths greater than `usize::MAX/4`!
        pub fn encoded_len(bytes: &[u8]) -> usize {
            crate::encoded_len(bytes, false)
        }
    }
}

/// Encode the input byte slice as Base64, writing the result into the provided
/// destination slice, and returning an ASCII-encoded string value.
#[inline(always)]
fn encode<'a>(
    src: &[u8],
    dst: &'a mut [u8],
    padded: bool,
    hi_bytes: (u8, u8),
) -> Result<&'a str, InvalidLengthError> {
    let elen = match encoded_len_inner(src.len(), padded) {
        Some(v) => v,
        None => return Err(InvalidLengthError),
    };

    if elen > dst.len() {
        return Err(InvalidLengthError);
    }

    let dst = &mut dst[..elen];

    let mut src_chunks = src.chunks_exact(3);
    let mut dst_chunks = dst.chunks_exact_mut(4);

    for (s, d) in (&mut src_chunks).zip(&mut dst_chunks) {
        encode_3bytes(s, d, hi_bytes);
    }

    let src_rem = src_chunks.remainder();

    if padded {
        if let Some(dst_rem) = dst_chunks.next() {
            let mut tmp = [0u8; 3];
            tmp[..src_rem.len()].copy_from_slice(&src_rem);
            encode_3bytes(&tmp, dst_rem, hi_bytes);

            let flag = src_rem.len() == 1;
            let mask = (flag as u8).wrapping_sub(1);
            dst_rem[2] = (dst_rem[2] & mask) | (PAD & !mask);
            dst_rem[3] = PAD;
        }
    } else {
        let dst_rem = dst_chunks.into_remainder();

        let mut tmp_in = [0u8; 3];
        let mut tmp_out = [0u8; 4];
        tmp_in[..src_rem.len()].copy_from_slice(src_rem);
        encode_3bytes(&tmp_in, &mut tmp_out, hi_bytes);
        dst_rem.copy_from_slice(&tmp_out[..dst_rem.len()]);
    }

    debug_assert!(str::from_utf8(dst).is_ok());

    // SAFETY: values written by `encode_3bytes` are valid one-byte UTF-8 chars
    Ok(unsafe { str::from_utf8_unchecked(dst) })
}

/// Encode the input byte slice as a Base64-encoded [`String`].
///
/// # Panics
/// If `input` length is greater than `usize::MAX/4`.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[inline(always)]
fn encode_string(input: &[u8], padded: bool, hi_bytes: (u8, u8)) -> String {
    let elen = encoded_len(input, padded);
    let mut dst = vec![0u8; elen];
    let res = encode(input, &mut dst, padded, hi_bytes).expect("encoding error");

    debug_assert_eq!(elen, res.len());
    debug_assert!(str::from_utf8(&dst).is_ok());

    // SAFETY: `dst` is fully written and contains only valid one-byte UTF-8 chars
    unsafe { String::from_utf8_unchecked(dst) }
}

/// Get the Base64-encoded length of the given byte slice.
///
/// WARNING: this function will return 0 for lengths greater than `usize::MAX/4`!
#[inline(always)]
const fn encoded_len(bytes: &[u8], padded: bool) -> usize {
    // TODO: replace with `unwrap_or` on stabilization
    match encoded_len_inner(bytes.len(), padded) {
        Some(v) => v,
        None => 0,
    }
}

#[inline(always)]
const fn encoded_len_inner(n: usize, padded: bool) -> Option<usize> {
    // TODO: replace with `checked_mul` and `map` on stabilization
    if n > usize::MAX / 4 {
        return None;
    }

    let q = 4 * n;

    if padded {
        Some(((q / 3) + 3) & !3)
    } else {
        Some((q / 3) + (q % 3 != 0) as usize)
    }
}

/// Decode the provided Base64 string into the provided destination buffer.
#[inline(always)]
fn decode(
    src: impl AsRef<[u8]>,
    dst: &mut [u8],
    padded: bool,
    hi_bytes: (u8, u8),
) -> Result<&[u8], Error> {
    let mut src = src.as_ref();

    let mut err = if padded {
        let (unpadded_len, e) = decode_padding(src)?;
        src = &src[..unpadded_len];
        e
    } else {
        0
    };

    let dlen = decoded_len(src.len());

    if dlen > dst.len() {
        return Err(Error::InvalidLength);
    }

    let dst = &mut dst[..dlen];

    let mut src_chunks = src.chunks_exact(4);
    let mut dst_chunks = dst.chunks_exact_mut(3);
    for (s, d) in (&mut src_chunks).zip(&mut dst_chunks) {
        err |= decode_3bytes(s, d, hi_bytes);
    }
    let src_rem = src_chunks.remainder();
    let dst_rem = dst_chunks.into_remainder();

    err |= !(src_rem.is_empty() || src_rem.len() >= 2) as i16;
    let mut tmp_out = [0u8; 3];
    let mut tmp_in = [b'A'; 4];
    tmp_in[..src_rem.len()].copy_from_slice(src_rem);
    err |= decode_3bytes(&tmp_in, &mut tmp_out, hi_bytes);
    dst_rem.copy_from_slice(&tmp_out[..dst_rem.len()]);

    if err == 0 {
        Ok(dst)
    } else {
        Err(Error::InvalidEncoding)
    }
}

/// Decode Base64-encoded string in-place.
#[inline(always)]
fn decode_in_place(
    mut buf: &mut [u8],
    padded: bool,
    hi_bytes: (u8, u8),
) -> Result<&[u8], InvalidEncodingError> {
    // TODO: eliminate unsafe code when compiler will be smart enough to
    // eliminate bound checks, see: https://github.com/rust-lang/rust/issues/80963
    let mut err = if padded {
        let (unpadded_len, e) = decode_padding(buf)?;
        buf = &mut buf[..unpadded_len];
        e
    } else {
        0
    };

    let dlen = decoded_len(buf.len());
    let full_chunks = buf.len() / 4;

    for chunk in 0..full_chunks {
        // SAFETY: `p3` and `p4` point inside `buf`, while they may overlap,
        // read and write are clearly separated from each other and done via
        // raw pointers.
        unsafe {
            debug_assert!(3 * chunk + 3 <= buf.len());
            debug_assert!(4 * chunk + 4 <= buf.len());

            let p3 = buf.as_mut_ptr().add(3 * chunk) as *mut [u8; 3];
            let p4 = buf.as_ptr().add(4 * chunk) as *const [u8; 4];

            let mut tmp_out = [0u8; 3];
            err |= decode_3bytes(&*p4, &mut tmp_out, hi_bytes);
            *p3 = tmp_out;
        }
    }

    let src_rem_pos = 4 * full_chunks;
    let src_rem_len = buf.len() - src_rem_pos;
    let dst_rem_pos = 3 * full_chunks;
    let dst_rem_len = dlen - dst_rem_pos;

    err |= !(src_rem_len == 0 || src_rem_len >= 2) as i16;
    let mut tmp_in = [b'A'; 4];
    tmp_in[..src_rem_len].copy_from_slice(&buf[src_rem_pos..]);
    let mut tmp_out = [0u8; 3];

    err |= decode_3bytes(&tmp_in, &mut tmp_out, hi_bytes);

    if err == 0 {
        // SAFETY: `dst_rem_len` is always smaller than 4, so we don't
        // read outside of `tmp_out`, write and the final slicing never go
        // outside of `buf`.
        unsafe {
            debug_assert!(dst_rem_pos + dst_rem_len <= buf.len());
            debug_assert!(dst_rem_len <= tmp_out.len());
            debug_assert!(dlen <= buf.len());

            core::ptr::copy_nonoverlapping(
                tmp_out.as_ptr(),
                buf.as_mut_ptr().add(dst_rem_pos),
                dst_rem_len,
            );
            Ok(buf.get_unchecked(..dlen))
        }
    } else {
        Err(InvalidEncodingError)
    }
}

/// Decode a Base64-encoded string into a byte vector.
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[inline(always)]
fn decode_vec(input: &str, padded: bool, hi_bytes: (u8, u8)) -> Result<Vec<u8>, Error> {
    let mut output = vec![0u8; decoded_len(input.len())];
    let len = decode(input, &mut output, padded, hi_bytes)?.len();

    if len <= output.len() {
        output.truncate(len);
        Ok(output)
    } else {
        Err(Error::InvalidLength)
    }
}

/// Get the length of the output from decoding the provided *unpadded*
/// Base64-encoded input (use [`unpadded_len_ct`] to compute this value for
/// a padded input)
///
/// Note that this function does not fully validate the Base64 is well-formed
/// and may return incorrect results for malformed Base64.
#[inline(always)]
fn decoded_len(input_len: usize) -> usize {
    // overflow-proof computation of `(3*n)/4`
    let k = input_len / 4;
    let l = input_len - 4 * k;
    3 * k + (3 * l) / 4
}

#[inline(always)]
fn encode_3bytes(src: &[u8], dst: &mut [u8], hi_bytes: (u8, u8)) {
    debug_assert_eq!(src.len(), 3);
    debug_assert!(dst.len() >= 4, "dst too short: {}", dst.len());

    let b0 = src[0] as i16;
    let b1 = src[1] as i16;
    let b2 = src[2] as i16;

    dst[0] = encode_6bits(b0 >> 2, hi_bytes);
    dst[1] = encode_6bits(((b0 << 4) | (b1 >> 4)) & 63, hi_bytes);
    dst[2] = encode_6bits(((b1 << 2) | (b2 >> 6)) & 63, hi_bytes);
    dst[3] = encode_6bits(b2 & 63, hi_bytes);
}

#[inline(always)]
fn encode_6bits(src: i16, hi_bytes: (u8, u8)) -> u8 {
    let hi_off = 0x1c + (hi_bytes.0 & 4);
    let mut diff = 0x41i16;

    diff += match_gt_ct(src, 25, 6);
    diff -= match_gt_ct(src, 51, 75);
    diff -= match_gt_ct(src, 61, hi_bytes.0 as i16 - hi_off as i16);
    diff += match_gt_ct(src, 62, hi_bytes.1 as i16 - hi_bytes.0 as i16 - 1);

    (src + diff) as u8
}

#[inline(always)]
fn decode_3bytes(src: &[u8], dst: &mut [u8], hi_bytes: (u8, u8)) -> i16 {
    debug_assert_eq!(src.len(), 4);
    debug_assert!(dst.len() >= 3, "dst too short: {}", dst.len());

    let c0 = decode_6bits(src[0], hi_bytes);
    let c1 = decode_6bits(src[1], hi_bytes);
    let c2 = decode_6bits(src[2], hi_bytes);
    let c3 = decode_6bits(src[3], hi_bytes);

    dst[0] = ((c0 << 2) | (c1 >> 4)) as u8;
    dst[1] = ((c1 << 4) | (c2 >> 2)) as u8;
    dst[2] = ((c2 << 6) | c3) as u8;

    ((c0 | c1 | c2 | c3) >> 8) & 1
}

#[inline(always)]
fn decode_6bits(src: u8, hi_bytes: (u8, u8)) -> i16 {
    let mut res: i16 = -1;
    res += match_range_ct(src, 0x41..0x5a, src as i16 - 64);
    res += match_range_ct(src, 0x61..0x7a, src as i16 - 70);
    res += match_range_ct(src, 0x30..0x39, src as i16 + 5);
    res += match_eq_ct(src, hi_bytes.0, 63);
    res + match_eq_ct(src, hi_bytes.1, 64)
}

/// Match that the given input is greater than the provided threshold.
#[inline(always)]
fn match_gt_ct(input: i16, threshold: u8, ret_on_match: i16) -> i16 {
    ((threshold as i16 - input) >> 8) & ret_on_match
}

/// Match that a byte falls within a provided range.
#[inline(always)]
fn match_range_ct(input: u8, range: Range<u8>, ret_on_match: i16) -> i16 {
    // Compute exclusive range from inclusive one
    let start = range.start as i16 - 1;
    let end = range.end as i16 + 1;

    (((start - input as i16) & (input as i16 - end)) >> 8) & ret_on_match
}

/// Match a a byte equals a specified value.
#[inline(always)]
fn match_eq_ct(input: u8, expected: u8, ret_on_match: i16) -> i16 {
    match_range_ct(input, expected..expected, ret_on_match)
}

/// Validate padding is well-formed and compute unpadded length.
///
/// Returns length-related errors eagerly as a [`Result`], and data-dependent
/// errors (i.e. malformed padding bytes) as `i16` to be combined with other
/// encoding-related errors prior to branching.
#[inline(always)]
fn decode_padding(input: &[u8]) -> Result<(usize, i16), InvalidEncodingError> {
    if input.len() % 4 != 0 {
        return Err(InvalidEncodingError);
    }

    let unpadded_len = match *input {
        [.., b0, b1] => {
            let pad_len = match_eq_ct(b0, PAD, 1) + match_eq_ct(b1, PAD, 1);
            input.len() - pad_len as usize
        }
        _ => input.len(),
    };

    let padding_len = input.len() - unpadded_len;

    let err = match *input {
        [.., b0] if padding_len == 1 => match_eq_ct(b0, PAD, 1) ^ 1,
        [.., b0, b1] if padding_len == 2 => (match_eq_ct(b0, PAD, 1) & match_eq_ct(b1, PAD, 1)) ^ 1,
        _ => {
            if padding_len == 0 {
                0
            } else {
                return Err(InvalidEncodingError);
            }
        }
    };

    Ok((unpadded_len, err))
}
