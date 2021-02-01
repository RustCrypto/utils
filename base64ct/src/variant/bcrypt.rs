//! bcrypt Base64 encoding.

use super::{Decode, Encode, Variant};

/// bcrypt Base64 encoding.
///
/// ```text
/// ./         [A-Z]      [a-z]     [0-9]
/// 0x2e-0x2f, 0x41-0x5a, 0x61-0x7a, 0x30-0x39
/// ```
pub struct Base64Bcrypt;

impl Variant for Base64Bcrypt {
    const PADDED: bool = false;
    const BASE: u8 = b'.';

    const DECODER: &'static [Decode] = &[
        Decode::Range(b'.'..b'/', -45),
        Decode::Range(b'A'..b'Z', -62),
        Decode::Range(b'a'..b'z', -68),
        Decode::Range(b'0'..b'9', 7),
    ];

    const ENCODER: &'static [Encode] = &[
        Encode::Apply(b'/', 17),
        Encode::Apply(b'Z', 6),
        Encode::Apply(b'z', -75),
    ];
}
