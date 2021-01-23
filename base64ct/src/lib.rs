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

//#![no_std]
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
                let mut tmp = [0u8; 3];
                tmp[..s.len()].copy_from_slice(&s);
                encode_3bytes(&tmp, d);

                d[3] = PAD;

                if s.len() == 1 {
                    d[2] = PAD;
                }
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
pub fn decode<'a>(src: &str, dst: &'a mut [u8], padded: bool) -> Result<&'a [u8], Error> {
    if padded {
        if let Some(c) = src.chars().last() {
            if c.is_whitespace() {
                return Err(Error::InvalidEncoding);
            }
        }

        if decoded_len(src, true) > dst.len() {
            return Err(Error::InvalidEncoding);
        }

        let src = src.as_bytes();

        let mut src_offset: usize = 0;
        let mut dst_offset: usize = 0;
        let mut src_length: usize = src.len();
        let mut err: isize = 0;

        while src_length > 4 {
            err |= decode_3bytes(
                &src[src_offset..(src_offset + 4)],
                &mut dst[dst_offset..(dst_offset + 3)],
            );
            src_offset += 4;
            dst_offset += 3;
            src_length -= 4;
        }

        if src_length > 0 {
            let mut i = 0;
            let mut tmp_out = [0u8; 3];
            let mut tmp_in = [b'A'; 4];

            while i < src_length && src[src_offset + i] != PAD {
                tmp_in[i] = src[src_offset + i];
                i += 1;
            }

            if i < 2 {
                err = 1;
            }

            src_length = i - 1;
            err |= decode_3bytes(&tmp_in, &mut tmp_out);

            dst[dst_offset..(dst_offset + src_length)].copy_from_slice(&tmp_out[..src_length]);
            dst_offset += i - 1;
        }

        if err == 0 {
            Ok(&dst[..dst_offset])
        } else {
            Err(Error::InvalidEncoding)
        }
    } else {
        let dlen = decoded_len(src, false);
        if dlen > dst.len() {
            return Err(Error::InvalidLength);
        }
        let src = src.as_bytes();
        let dst = &mut dst[..dlen];

        let mut err: isize = 0;

        let mut src_chunks = src.chunks_exact(4);
        let mut dst_chunks = dst.chunks_exact_mut(3);
        for (s, d) in (&mut src_chunks).zip(&mut dst_chunks) {
            err |= decode_3bytes(s, d);
        }
        let src_rem = src_chunks.remainder();
        let dst_rem = dst_chunks.into_remainder();

        err |= !(src_rem.is_empty() || src_rem.len() >= 2) as isize;
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
}

/// Decode unpadded Base64-encoded string in-place.
// TODO(tarcieri): support for padded Base64
pub fn decode_in_place_unpadded(buf: &mut [u8]) -> Result<&[u8], InvalidEncodingError> {
    // TODO: eliminate unsafe code when compiler will be smart enough to
    // eliminate bound checks, see: https://github.com/rust-lang/rust/issues/80963
    let mut err: isize = 0;
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

    let dlen = decoded_len_inner_unpadded(buf.len());
    let src_rem_pos = 4 * full_chunks;
    let src_rem_len = buf.len() - src_rem_pos;
    let dst_rem_pos = 3 * full_chunks;
    let dst_rem_len = dlen - dst_rem_pos;

    err |= !(src_rem_len == 0 || src_rem_len >= 2) as isize;
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
    let dlen = decoded_len(input, padded);
    let mut output = vec![0u8; dlen];
    let res = decode(input, &mut output, padded)?;
    debug_assert_eq!(dlen, res.len());
    Ok(output)
}

/// Get the length of the output from decoding the provided Base64-encoded input.
pub const fn decoded_len(input: &str, padded: bool) -> usize {
    if padded {
        let bytes = input.as_bytes();

        if bytes.is_empty() {
            return 0;
        }

        let mut i = bytes.len() - 1;
        let mut pad_count: usize = 0;

        while i > 0 && bytes[i] == PAD {
            pad_count += 1;
            i -= 1;
        }

        ((bytes.len() - pad_count) * 3) / 4
    } else {
        decoded_len_inner_unpadded(input.len())
    }
}

#[inline(always)]
const fn decoded_len_inner_unpadded(n: usize) -> usize {
    // branchless, overflow-proof computation of `(3*n)/4`
    let k = n / 4;
    let l = n - 4 * k;
    3 * k + (3 * l) / 4
}

// Base64 character set:
// [A-Z]      [a-z]      [0-9]      +     /
// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f

// TODO(tarcieri): support for Base64url

#[inline(always)]
pub(crate) fn encode_3bytes(src: &[u8], dst: &mut [u8]) {
    debug_assert_eq!(src.len(), 3);
    debug_assert!(dst.len() >= 4, "dst too short: {}", dst.len());

    let b0 = src[0] as isize;
    let b1 = src[1] as isize;
    let b2 = src[2] as isize;

    dst[0] = encode_6bits(b0 >> 2);
    dst[1] = encode_6bits(((b0 << 4) | (b1 >> 4)) & 63);
    dst[2] = encode_6bits(((b1 << 2) | (b2 >> 6)) & 63);
    dst[3] = encode_6bits(b2 & 63);
}

#[inline(always)]
pub(crate) fn encode_6bits(src: isize) -> u8 {
    let mut diff = 0x41isize;

    // if (in > 25) diff += 0x61 - 0x41 - 26; // 6
    diff += ((25isize - src) >> 8) & 6;

    // if (in > 51) diff += 0x30 - 0x61 - 26; // -75
    diff -= ((51isize - src) >> 8) & 75;

    // if (in > 61) diff += 0x2b - 0x30 - 10; // -15
    diff -= ((61isize - src) >> 8) & 15;

    // if (in > 62) diff += 0x2f - 0x2b - 1; // 3
    diff += ((62isize - src) >> 8) & 3;

    (src + diff) as u8
}

#[inline(always)]
pub(crate) fn decode_3bytes(src: &[u8], dst: &mut [u8]) -> isize {
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
pub(crate) fn decode_6bits(src: u8) -> isize {
    let ch = src as isize;
    let mut ret: isize = -1;

    // if (ch > 0x40 && ch < 0x5b) ret += ch - 0x41 + 1; // -64
    ret += (((64isize - ch) & (ch - 91isize)) >> 8) & (ch - 64isize);

    // if (ch > 0x60 && ch < 0x7b) ret += ch - 0x61 + 26 + 1; // -70
    ret += (((96isize - ch) & (ch - 123isize)) >> 8) & (ch - 70isize);

    // if (ch > 0x2f && ch < 0x3a) ret += ch - 0x30 + 52 + 1; // 5
    ret += (((47isize - ch) & (ch - 58isize)) >> 8) & (ch + 5isize);

    // if (ch == 0x2b) ret += 62 + 1;
    ret += (((42isize - ch) & (ch - 44isize)) >> 8) & 63;

    // if (ch == 0x2f) ret += 63 + 1;
    ret + ((((46isize - ch) & (ch - 48isize)) >> 8) & 64)
}
