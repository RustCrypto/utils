//! This crate provides the [`hex!`] macro for converting hexadecimal string literals
//! to a byte array at compile time.
//!
//! It accepts the following characters in the input string:
//!
//! - `'0'...'9'`, `'a'...'f'`, `'A'...'F'` — hex characters which will be used
//!     in construction of the output byte array
//! - `' '`, `'\r'`, `'\n'`, `'\t'` — formatting characters which will be
//!     ignored
//!
//! # Examples
//! ```
//! # #[macro_use] extern crate hex_literal;
//! // The macro can be used in const contexts
//! const DATA: [u8; 4] = hex!("01020304");
//! # fn main() {
//! assert_eq!(DATA, [1, 2, 3, 4]);
//!
//! // Both upper and lower hex values are supported
//! assert_eq!(hex!("a1 b2 c3 d4"), [0xA1, 0xB2, 0xC3, 0xD4]);
//! assert_eq!(hex!("E5 E6 90 92"), [0xE5, 0xE6, 0x90, 0x92]);
//! assert_eq!(hex!("0a0B 0C0d"), [10, 11, 12, 13]);
//!
//! // Multi-line literals
//! let bytes1 = hex!("
//!     00010203 04050607
//!     08090a0b 0c0d0e0f
//! ");
//! assert_eq!(bytes1, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
//!
//! // It's possible to use several literals (results will be concatenated)
//! let bytes2 = hex!(
//!     "00010203 04050607" // first half
//!     "08090a0b" /* block comment */ "0c0d0e0f" // second half
//! );
//! assert_eq!(bytes1, bytes2);
//! # }
//! ```
//!
//! Using an unsupported character inside literals will result
//! in a compilation error:
//! ```compile_fail
//! # use hex_literal::hex;
//! hex!("АА"); // Cyrillic "А"
//! hex!("11　22"); // Japanese space
//! hex!("0123 // Сomments inside literals are not supported");
//! ```
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

const fn next_hex_char(string: &[u8], mut pos: usize) -> Option<(u8, usize)> {
    while pos < string.len() {
        let raw_val = string[pos];
        pos += 1;
        let val = match raw_val {
            b'0'..=b'9' => raw_val - 48,
            b'A'..=b'F' => raw_val - 55,
            b'a'..=b'f' => raw_val - 87,
            b' ' | b'\r' | b'\n' | b'\t' => continue,
            0..=127 => panic!("Encountered invalid ASCII character"),
            _ => panic!("Encountered non-ASCII character"),
        };
        return Some((val, pos));
    }
    None
}

const fn next_byte(string: &[u8], pos: usize) -> Option<(u8, usize)> {
    let (half1, pos) = match next_hex_char(string, pos) {
        Some(v) => v,
        None => return None,
    };
    let (half2, pos) = match next_hex_char(string, pos) {
        Some(v) => v,
        None => panic!("Odd number of hex characters"),
    };
    Some(((half1 << 4) + half2, pos))
}

#[doc(hidden)]
pub const fn len(strings: &[&[u8]]) -> usize {
    let mut i = 0;
    let mut len = 0;
    while i < strings.len() {
        let mut pos = 0;
        while let Some((_, new_pos)) = next_byte(strings[i], pos) {
            len += 1;
            pos = new_pos;
        }
        i += 1;
    }
    len
}

#[doc(hidden)]
pub const fn decode<const LEN: usize>(strings: &[&[u8]]) -> [u8; LEN] {
    let mut i = 0;
    let mut buf = [0u8; LEN];
    let mut buf_pos = 0;
    while i < strings.len() {
        let mut pos = 0;
        while let Some((byte, new_pos)) = next_byte(strings[i], pos) {
            buf[buf_pos] = byte;
            buf_pos += 1;
            pos = new_pos;
        }
        i += 1;
    }
    if LEN != buf_pos {
        panic!("Length mismatch. Please report this bug.");
    }
    buf
}

/// Macro for converting sequence of string literals containing hex-encoded data
/// into an array of bytes.
#[macro_export]
macro_rules! hex {
    ($($s:literal)*) => {{
        const STRINGS: &[&'static [u8]] = &[$($s.as_bytes(),)*];
        const LEN: usize = $crate::len(STRINGS);
        $crate::decode::<LEN>(STRINGS)
    }};
}
