//! `crypt(3)` Base64 encoding.

use super::{Decode, Encode, Variant};

/// `crypt(3)` Base64 encoding.
///
/// ```text
/// [.-9]      [A-Z]      [a-z]
/// 0x2e-0x39, 0x41-0x5a, 0x61-0x7a
/// ```
pub struct Base64Crypt;

impl Variant for Base64Crypt {
    const PADDED: bool = false;
    const BASE: u8 = b'.';

    const DECODER: &'static [Decode] = &[
        Decode::Range(b'.'..b'9', -45),
        Decode::Range(b'A'..b'Z', -52),
        Decode::Range(b'a'..b'z', -58),
    ];

    const ENCODER: &'static [Encode] = &[Encode::Apply(b'9', 7), Encode::Apply(b'Z', 6)];
}
