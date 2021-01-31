//! Standard Base64 encoding.

use crate::{
    encoding::{match_eq_ct, match_gt_ct, match_range_ct},
    variant::Variant,
};

/// Standard Base64 encoding with `=` padding.
///
/// ```text
/// [A-Z]      [a-z]      [0-9]      +     /
/// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f
/// ```
pub struct Base64;

impl Variant for Base64 {
    const PADDED: bool = true;

    #[inline]
    fn decode_6bits(src: u8) -> i16 {
        decode_6bits(src)
    }

    #[inline]
    fn encode_6bits(src: i16) -> u8 {
        encode_6bits(src)
    }
}

/// Standard Base64 encoding *without* padding.
///
/// ```text
/// [A-Z]      [a-z]      [0-9]      +     /
/// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f
/// ```
pub struct Base64Unpadded;

impl Variant for Base64Unpadded {
    const PADDED: bool = false;

    #[inline]
    fn decode_6bits(src: u8) -> i16 {
        decode_6bits(src)
    }

    #[inline]
    fn encode_6bits(src: i16) -> u8 {
        encode_6bits(src)
    }
}

#[inline(always)]
fn decode_6bits(src: u8) -> i16 {
    let mut res: i16 = -1;
    res += match_range_ct(src, b'A'..b'Z', src as i16 - 64);
    res += match_range_ct(src, b'a'..b'z', src as i16 - 70);
    res += match_range_ct(src, b'0'..b'9', src as i16 + 5);
    res += match_eq_ct(src, b'+', 63);
    res + match_eq_ct(src, b'/', 64)
}

#[inline(always)]
fn encode_6bits(src: i16) -> u8 {
    let mut diff = b'A' as i16;
    diff += match_gt_ct(src, 25, 6);
    diff -= match_gt_ct(src, 51, 75);
    diff -= match_gt_ct(src, 61, b'+' as i16 - 0x1c);
    diff += match_gt_ct(src, 62, b'/' as i16 - b'+' as i16 - 1);
    (src + diff) as u8
}
