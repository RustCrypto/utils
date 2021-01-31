//! bcrypt Base64 encoding.

use crate::{
    encoding::{match_gt_ct, match_range_ct},
    variant::Variant,
};

/// bcrypt Base64 encoding.
///
/// ```text
/// ./         [A-Z]      [a-z]     [0-9]
/// 0x2e-0x2f, 0x41-0x5a, 0x61-0x7a, 0x30-0x39
/// ```
pub struct Base64Bcrypt;

impl Variant for Base64Bcrypt {
    const PADDED: bool = false;

    #[inline]
    fn decode_6bits(src: u8) -> i16 {
        let mut res: i16 = -1;
        res += match_range_ct(src, b'.'..b'/', src as i16 - 45);
        res += match_range_ct(src, b'A'..b'Z', src as i16 - 62);
        res += match_range_ct(src, b'a'..b'z', src as i16 - 68);
        res + match_range_ct(src, b'0'..b'9', src as i16 + 7)
    }

    #[inline]
    fn encode_6bits(mut src: i16) -> u8 {
        src += 0x2e;
        src += match_gt_ct(src, 0x2f, 17);
        src += match_gt_ct(src, 0x5a, 6);
        src -= match_gt_ct(src, 0x7a, 75);
        src as u8
    }
}
