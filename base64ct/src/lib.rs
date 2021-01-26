//! Pure Rust implementation of Base64 encoding ([RFC 4648, section 4]).
//!
//! # Implementation
//!
//! Implemented without data-dependent branches or lookup tables, thereby
//! providing portable "best effort" constant-time operation.
//!
//! Adapted from the following constant-time C++ implementation of Base64:
//!
//! <https://github.com/Sc00bz/ConstTimeEncoding/blob/master/base64.cpp>
//!
//! Copyright (c) 2014 Steve "Sc00bz" Thomas (steve at tobtu dot com).
//! Derived code is dual licensed MIT + Apache 2 (with permission from Sc00bz).
//!
//! [PHC string format]: https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md
//! [RFC 4648, section 4]: https://tools.ietf.org/html/rfc4648#section-4

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/base64ct/0.0.0"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod errors;

pub use errors::{Error, InvalidEncodingError, InvalidLengthError};

use core::str;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Padding character
const PAD: u8 = b'=';

/// Encode the input byte slice as Base64, writing the result into the provided
/// destination slice, and returning an ASCII-encoded string value.
#[inline]
pub fn encode<'a>(
    src: &[u8],
    dst: &'a mut [u8],
    padded: bool,
) -> Result<&'a str, InvalidLengthError> {
    let elen = match encoded_len_inner(src.len(), padded) {
        Some(v) => v,
        None => return Err(InvalidLengthError),
    };

    if elen > dst.len() {
        return Err(InvalidLengthError);
    }

    let dst = &mut dst[..elen];

    if padded {
        for (s, d) in src.chunks(3).zip(dst.chunks_mut(4)) {
            if s.len() == 3 {
                encode_3bytes(s, d);
            } else {
                encode_3bytes_padded(s, d);
            }
        }
    } else {
        let mut src_chunks = src.chunks_exact(3);
        let mut dst_chunks = dst.chunks_exact_mut(4);

        for (s, d) in (&mut src_chunks).zip(&mut dst_chunks) {
            encode_3bytes(s, d);
        }

        let src_rem = src_chunks.remainder();
        let dst_rem = dst_chunks.into_remainder();

        let mut tmp_in = [0u8; 3];
        let mut tmp_out = [0u8; 4];
        tmp_in[..src_rem.len()].copy_from_slice(src_rem);
        encode_3bytes(&tmp_in, &mut tmp_out);
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
pub fn encode_string(input: &[u8], padded: bool) -> String {
    let elen = encoded_len(input, padded);
    let mut dst = vec![0u8; elen];
    let res = encode(input, &mut dst, padded).expect("encoding error");

    debug_assert_eq!(elen, res.len());
    debug_assert!(str::from_utf8(&dst).is_ok());

    // SAFETY: `dst` is fully written and contains only valid one-byte UTF-8 chars
    unsafe { String::from_utf8_unchecked(dst) }
}

/// Get the Base64-encoded length of the given byte slice.
///
/// WARNING: this function will return 0 for lengths greater than `usize::MAX/4`!
#[inline]
pub const fn encoded_len(bytes: &[u8], padded: bool) -> usize {
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
pub fn decode(src: impl AsRef<[u8]>, dst: &mut [u8], padded: bool) -> Result<&[u8], Error> {
    let mut src = src.as_ref();
    let mut err = 0;

    if padded {
        err = validate_padding(src)?;
        src = &src[..unpadded_len_ct(src)];
    };

    let dlen = decoded_len(src.len());

    if dlen > dst.len() {
        return Err(Error::InvalidLength);
    }

    let dst = &mut dst[..dlen];

    let mut src_chunks = src.chunks_exact(4);
    let mut dst_chunks = dst.chunks_exact_mut(3);
    for (s, d) in (&mut src_chunks).zip(&mut dst_chunks) {
        err |= decode_3bytes(s, d);
    }
    let src_rem = src_chunks.remainder();
    let dst_rem = dst_chunks.into_remainder();

    err |= !(src_rem.is_empty() || src_rem.len() >= 2) as i16;
    let mut tmp_out = [0u8; 3];
    let mut tmp_in = [b'A'; 4];
    tmp_in[..src_rem.len()].copy_from_slice(src_rem);
    err |= decode_3bytes(&tmp_in, &mut tmp_out);
    dst_rem.copy_from_slice(&tmp_out[..dst_rem.len()]);

    if err == 0 {
        Ok(dst)
    } else {
        Err(Error::InvalidEncoding)
    }
}

/// Decode Base64-encoded string in-place.
pub fn decode_in_place(mut buf: &mut [u8], padded: bool) -> Result<&[u8], InvalidEncodingError> {
    // TODO: eliminate unsafe code when compiler will be smart enough to
    // eliminate bound checks, see: https://github.com/rust-lang/rust/issues/80963
    let mut err = 0;

    if padded {
        err = validate_padding(buf)?;
        let unpadded_len = unpadded_len_ct(buf);
        buf = &mut buf[..unpadded_len];
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
            err |= decode_3bytes(&*p4, &mut tmp_out);
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

    err |= decode_3bytes(&tmp_in, &mut tmp_out);

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
pub fn decode_vec(input: &str, padded: bool) -> Result<Vec<u8>, Error> {
    let slen = if padded {
        unpadded_len_ct(input.as_bytes())
    } else {
        input.as_bytes().len()
    };

    let dlen = decoded_len(slen);

    let mut output = vec![0u8; dlen];
    let res = decode(input, &mut output, padded)?;
    debug_assert_eq!(dlen, res.len());
    Ok(output)
}

/// Get the length of the output from decoding the provided *unpadded*
/// Base64-encoded input (use [`unpadded_len_ct`] to compute this value for
/// a padded input)
///
/// Note that this function does not fully validate the Base64 is well-formed
/// and may return incorrect results for malformed Base64.
#[inline]
const fn decoded_len(input_len: usize) -> usize {
    // overflow-proof computation of `(3*n)/4`
    let k = input_len / 4;
    let l = input_len - 4 * k;
    3 * k + (3 * l) / 4
}

// Base64 character set:
// [A-Z]      [a-z]      [0-9]      +     /
// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f

// TODO(tarcieri): support for Base64url

#[inline(always)]
fn encode_3bytes(src: &[u8], dst: &mut [u8]) {
    debug_assert_eq!(src.len(), 3);
    debug_assert!(dst.len() >= 4, "dst too short: {}", dst.len());

    let b0 = src[0] as i16;
    let b1 = src[1] as i16;
    let b2 = src[2] as i16;

    dst[0] = encode_6bits(b0 >> 2);
    dst[1] = encode_6bits(((b0 << 4) | (b1 >> 4)) & 63);
    dst[2] = encode_6bits(((b1 << 2) | (b2 >> 6)) & 63);
    dst[3] = encode_6bits(b2 & 63);
}

#[inline(always)]
fn encode_3bytes_padded(src: &[u8], dst: &mut [u8]) {
    let mut tmp = [0u8; 3];
    tmp[..src.len()].copy_from_slice(&src);
    encode_3bytes(&tmp, dst);

    dst[3] = PAD;

    if src.len() == 1 {
        dst[2] = PAD;
    }
}

#[inline(always)]
fn encode_6bits(src: i16) -> u8 {
    let mut diff = 0x41i16;

    // if (in > 25) diff += 0x61 - 0x41 - 26; // 6
    diff += ((25i16 - src) >> 8) & 6;

    // if (in > 51) diff += 0x30 - 0x61 - 26; // -75
    diff -= ((51i16 - src) >> 8) & 75;

    // if (in > 61) diff += 0x2b - 0x30 - 10; // -15
    diff -= ((61i16 - src) >> 8) & 15;

    // if (in > 62) diff += 0x2f - 0x2b - 1; // 3
    diff += ((62i16 - src) >> 8) & 3;

    (src + diff) as u8
}

#[inline(always)]
fn decode_3bytes(src: &[u8], dst: &mut [u8]) -> i16 {
    debug_assert_eq!(src.len(), 4);
    debug_assert!(dst.len() >= 3, "dst too short: {}", dst.len());

    let c0 = decode_6bits(src[0]);
    let c1 = decode_6bits(src[1]);
    let c2 = decode_6bits(src[2]);
    let c3 = decode_6bits(src[3]);

    dst[0] = ((c0 << 2) | (c1 >> 4)) as u8;
    dst[1] = ((c1 << 4) | (c2 >> 2)) as u8;
    dst[2] = ((c2 << 6) | c3) as u8;

    ((c0 | c1 | c2 | c3) >> 8) & 1
}

#[inline(always)]
fn decode_6bits(src: u8) -> i16 {
    let mut res: i16 = -1;

    // if (byte > 0x40 && byte < 0x5b) res += byte - 0x41 + 1; // -64
    res += match_byte_range_ct(src, 0x40, 0x5b, src as i16 - 64);

    // if (byte > 0x60 && byte < 0x7b) res += byte - 0x61 + 26 + 1; // -70
    res += match_byte_range_ct(src, 0x60, 0x7b, src as i16 - 70);

    // if (byte > 0x2f && byte < 0x3a) res += byte - 0x30 + 52 + 1; // 5
    res += match_byte_range_ct(src, 0x2f, 0x3a, src as i16 + 5);

    // if (byte == 0x2b) res += 62 + 1;
    res += match_byte_ct(src, 0x2b, 63);

    // if (byte == 0x2f) res += 63 + 1;
    res + match_byte_ct(src, 0x2f, 64)
}

/// Pseudo-branch operation
#[inline(always)]
fn match_byte_range_ct(input: u8, lo: u8, hi: u8, ret_on_match: i16) -> i16 {
    (((lo as i16 - input as i16) & (input as i16 - hi as i16)) >> 8) & ret_on_match
}

/// Match a specific byte value
#[inline(always)]
fn match_byte_ct(input: u8, expected: u8, ret_on_match: i16) -> i16 {
    match_byte_range_ct(input, expected - 1, expected + 1, ret_on_match)
}

/// Compute the length of the unpadded portion of a Base64-encoded string
/// without data-dependent branches
fn unpadded_len_ct(input: &[u8]) -> usize {
    match *input {
        [.., b0, b1] => {
            let pad_len = match_byte_ct(b0, PAD, 1) + match_byte_ct(b1, PAD, 1);
            input.len() - pad_len as usize
        }
        _ => input.len(),
    }
}

/// Validate padding is well-formed.
///
/// Returns length-related errors eagerly as a [`Result`]], and data-dependent
/// errors (i.e. malformed padding bytes) as `i16` to be combined with other
/// encoding-related errors prior to branching.
fn validate_padding(input: &[u8]) -> Result<i16, InvalidEncodingError> {
    if input.len() % 4 != 0 {
        return Err(InvalidEncodingError);
    }

    let padding_len = input.len() - unpadded_len_ct(input);

    match *input {
        [.., b0] if padding_len == 1 => Ok(match_byte_ct(b0, PAD, 1) ^ 1),
        [.., b0, b1] if padding_len == 2 => {
            Ok((match_byte_ct(b0, PAD, 1) & match_byte_ct(b1, PAD, 1)) ^ 1)
        }
        _ => {
            if padding_len == 0 {
                Ok(0)
            } else {
                Err(InvalidEncodingError)
            }
        }
    }
}
