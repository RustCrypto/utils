//! `crypt(3)` Base64 encoding.

use crate::{
    encoding::{match_gt_ct, match_range_ct},
    variant::Variant,
};

/// `crypt(3)` Base64 encoding.
///
/// ```text
/// [.-9]      [A-Z]      [a-z]
/// 0x2e-0x39, 0x41-0x5a, 0x61-0x7a
/// ```
pub struct Base64Crypt;

impl Variant for Base64Crypt {
    const PADDED: bool = false;

    #[inline]
    fn decode_6bits(src: u8) -> i16 {
        let mut res: i16 = -1;
        res += match_range_ct(src, b'.'..b'9', src as i16 - 45);
        res += match_range_ct(src, b'A'..b'Z', src as i16 - 52);
        res + match_range_ct(src, b'a'..b'z', src as i16 - 58)
    }

    #[inline]
    fn encode_6bits(mut src: i16) -> u8 {
        src += 0x2e;
        src += match_gt_ct(src, 0x39, 7);
        src += match_gt_ct(src, 0x5a, 6);
        src as u8
    }
}
