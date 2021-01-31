//! Base64 decoder.

use crate::{Error, InvalidEncodingError, PAD};
use core::ops::Range;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Decode the provided Base64 string into the provided destination buffer.
#[inline(always)]
pub(crate) fn decode<F>(
    src: impl AsRef<[u8]>,
    dst: &mut [u8],
    padded: bool,
    decode_6bits: F,
) -> Result<&[u8], Error>
where
    F: Fn(u8) -> i16 + Copy,
{
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
        err |= decode_3bytes(s, d, decode_6bits);
    }
    let src_rem = src_chunks.remainder();
    let dst_rem = dst_chunks.into_remainder();

    err |= !(src_rem.is_empty() || src_rem.len() >= 2) as i16;
    let mut tmp_out = [0u8; 3];
    let mut tmp_in = [b'A'; 4];
    tmp_in[..src_rem.len()].copy_from_slice(src_rem);
    err |= decode_3bytes(&tmp_in, &mut tmp_out, decode_6bits);
    dst_rem.copy_from_slice(&tmp_out[..dst_rem.len()]);

    if err == 0 {
        Ok(dst)
    } else {
        Err(Error::InvalidEncoding)
    }
}

/// Decode Base64-encoded string in-place.
#[inline(always)]
pub(crate) fn decode_in_place<F>(
    mut buf: &mut [u8],
    padded: bool,
    decode_6bits: F,
) -> Result<&[u8], InvalidEncodingError>
where
    F: Fn(u8) -> i16 + Copy,
{
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
            err |= decode_3bytes(&*p4, &mut tmp_out, decode_6bits);
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

    err |= decode_3bytes(&tmp_in, &mut tmp_out, decode_6bits);

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
pub(crate) fn decode_vec<F>(input: &str, padded: bool, decode_6bits: F) -> Result<Vec<u8>, Error>
where
    F: Fn(u8) -> i16 + Copy,
{
    let mut output = vec![0u8; decoded_len(input.len())];
    let len = decode(input, &mut output, padded, decode_6bits)?.len();

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
fn decode_3bytes<F>(src: &[u8], dst: &mut [u8], decode_6bits: F) -> i16
where
    F: Fn(u8) -> i16 + Copy,
{
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

/// Match that a byte falls within a provided range.
#[inline(always)]
pub(crate) fn match_range_ct(input: u8, range: Range<u8>, ret_on_match: i16) -> i16 {
    // Compute exclusive range from inclusive one
    let start = range.start as i16 - 1;
    let end = range.end as i16 + 1;

    (((start - input as i16) & (input as i16 - end)) >> 8) & ret_on_match
}

/// Match a a byte equals a specified value.
#[inline(always)]
pub(crate) fn match_eq_ct(input: u8, expected: u8, ret_on_match: i16) -> i16 {
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
