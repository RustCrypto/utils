#![no_std]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/6ee8e381/logo.svg"
)]

const fn next_hex_char(string: &[u8], mut pos: usize, extra_padding: &[u8]) -> Option<(u8, usize)> {
    while pos < string.len() {
        let raw_val = string[pos];
        pos += 1;
        let val = match raw_val {
            b'0'..=b'9' => raw_val - 48,
            b'A'..=b'F' => raw_val - 55,
            b'a'..=b'f' => raw_val - 87,
            b' ' | b'\r' | b'\n' | b'\t' => continue,
            b if byte_slice_contains(extra_padding, b) => continue,
            0..=127 => panic!("Encountered invalid ASCII character"),
            _ => panic!("Encountered non-ASCII character"),
        };
        return Some((val, pos));
    }
    None
}

const fn next_byte(string: &[u8], pos: usize, extra_padding: &[u8]) -> Option<(u8, usize)> {
    let (half1, pos) = match next_hex_char(string, pos, extra_padding) {
        Some(v) => v,
        None => return None,
    };
    let (half2, pos) = match next_hex_char(string, pos, extra_padding) {
        Some(v) => v,
        None => panic!("Odd number of hex characters"),
    };
    Some(((half1 << 4) + half2, pos))
}

/// Compute length of a byte array which will be decoded from the strings.
///
/// This function is an implementation detail and SHOULD NOT be called directly!
#[doc(hidden)]
pub const fn len(strings: &[&[u8]], extra_padding: &[u8]) -> usize {
    let mut i = 0;
    let mut len = 0;
    while i < strings.len() {
        let mut pos = 0;
        while let Some((_, new_pos)) = next_byte(strings[i], pos, extra_padding) {
            len += 1;
            pos = new_pos;
        }
        i += 1;
    }
    len
}

/// Decode hex strings into a byte array of pre-computed length.
///
/// This function is an implementation detail and SHOULD NOT be called directly!
#[doc(hidden)]
pub const fn decode<const LEN: usize>(
    strings: &[&[u8]],
    extra_padding: &[u8],
) -> Option<[u8; LEN]> {
    let mut string_pos = 0;
    let mut buf = [0u8; LEN];
    let mut buf_pos = 0;
    while string_pos < strings.len() {
        let mut pos = 0;
        let string = &strings[string_pos];
        string_pos += 1;

        while let Some((byte, new_pos)) = next_byte(string, pos, extra_padding) {
            buf[buf_pos] = byte;
            buf_pos += 1;
            pos = new_pos;
        }
    }
    if LEN == buf_pos { Some(buf) } else { None }
}

/// Converts a sequence of hexadecimal string literals to a byte array at compile time.
///
/// See the crate-level docs for more information.
#[macro_export]
macro_rules! hex {
    ($($s:literal)*) => {
        ::hex_literal::hex_custom_padding!("", $($s)*)
    }
}

#[macro_export]
macro_rules! hex_custom_padding {
    ($padding:literal, $($s:literal)*) => {{
        const PADDING: &'static [u8] = $padding.as_bytes();
        const STRINGS: &[&'static [u8]] = &[$($s.as_bytes(),)*];
        const {
            $crate::decode::<{ $crate::len(STRINGS, PADDING) }>(STRINGS, PADDING)
                .expect("Output array length should be correct")
        }
    }};
}

const fn byte_slice_contains(haystack: &[u8], needle: u8) -> bool {
    let mut i = 0;
    while i < haystack.len() {
        if haystack[i] == needle {
            return true;
        }
        i += 1;
    }
    false
}
