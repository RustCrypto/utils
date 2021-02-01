//! Standard Base64 encoding.

use super::{Decode, Encode, Variant};

/// Standard Base64 encoding with `=` padding.
///
/// ```text
/// [A-Z]      [a-z]      [0-9]      +     /
/// 0x41-0x5a, 0x61-0x7a, 0x30-0x39, 0x2b, 0x2f
/// ```
pub struct Base64;

impl Variant for Base64 {
    const PADDED: bool = true;
    const BASE: u8 = b'A';
    const DECODER: &'static [Decode] = DECODER;
    const ENCODER: &'static [Encode] = ENCODER;
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
    const BASE: u8 = b'A';
    const DECODER: &'static [Decode] = DECODER;
    const ENCODER: &'static [Encode] = ENCODER;
}

/// Standard Base64 decoder
const DECODER: &[Decode] = &[
    Decode::Range(b'A'..b'Z', -64),
    Decode::Range(b'a'..b'z', -70),
    Decode::Range(b'0'..b'9', 5),
    Decode::Eq(b'+', 63),
    Decode::Eq(b'/', 64),
];

/// Standard Base64 encoder
const ENCODER: &[Encode] = &[
    Encode::Diff(25, 6),
    Encode::Diff(51, -75),
    Encode::Diff(61, -(b'+' as i16 - 0x1c)),
    Encode::Diff(62, b'/' as i16 - b'+' as i16 - 1),
];
