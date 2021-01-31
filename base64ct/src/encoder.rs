//! Base64 encoder.

use crate::{InvalidLengthError, PAD};
use core::str;

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Get the Base64-encoded length of the given byte slice.
///
/// WARNING: this function will return 0 for lengths greater than `usize::MAX/4`!
#[inline(always)]
pub(crate) const fn encoded_len(bytes: &[u8], padded: bool) -> usize {
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

/// Encode the input byte slice as Base64, writing the result into the provided
/// destination slice, and returning an ASCII-encoded string value.
#[inline(always)]
pub(crate) fn encode<'a>(
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
pub(crate) fn encode_string(input: &[u8], padded: bool, hi_bytes: (u8, u8)) -> String {
    let elen = encoded_len_inner(input.len(), padded).expect("input is too big");
    let mut dst = vec![0u8; elen];
    let res = encode(input, &mut dst, padded, hi_bytes).expect("encoding error");

    debug_assert_eq!(elen, res.len());
    debug_assert!(str::from_utf8(&dst).is_ok());

    // SAFETY: `dst` is fully written and contains only valid one-byte UTF-8 chars
    unsafe { String::from_utf8_unchecked(dst) }
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

/// Match that the given input is greater than the provided threshold.
#[inline(always)]
fn match_gt_ct(input: i16, threshold: u8, ret_on_match: i16) -> i16 {
    ((threshold as i16 - input) >> 8) & ret_on_match
}
