//! This crate provides `hex!` macro for converting hexadecimal string literal
//! to byte array at compile time.
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
//! const DATA: [u8; 4] = hex!("01020304");
//!
//! # fn main() {
//! assert_eq!(DATA, [1, 2, 3, 4]);
//! assert_eq!(hex!("a1 b2 c3 d4"), [0xA1, 0xB2, 0xC3, 0xD4]);
//! assert_eq!(hex!("E5 E6 90 92"), [0xE5, 0xE6, 0x90, 0x92]);
//! assert_eq!(hex!("0a0B 0C0d"), [10, 11, 12, 13]);
//! let bytes = hex!("
//!     00010203 04050607
//!     08090a0b 0c0d0e0f
//! ");
//! assert_eq!(bytes, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
//! # }
//! ```
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/hex-literal/0.3.1"
)]

extern crate proc_macro;

use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};
use std::iter::FromIterator;

/// Strips any outer `Delimiter::None` groups from the input,
/// returning a `TokenStream` consisting of the innermost
/// non-empty-group `TokenTree`.
/// This is used to handle a proc macro being invoked
/// by a `macro_rules!` expansion.
/// See https://github.com/rust-lang/rust/issues/72545 for background
fn ignore_groups(mut input: TokenStream) -> TokenStream {
    let mut tokens = input.clone().into_iter();
    loop {
        if let Some(TokenTree::Group(group)) = tokens.next() {
            if group.delimiter() == Delimiter::None {
                input = group.stream();
                continue;
            }
        }
        return input;
    }
}

struct TokenTreeIter {
    buf: Vec<u8>,
    pos: usize,
    is_punct: bool,
}

impl TokenTreeIter {
    fn new(input: TokenStream) -> Self {
        let mut ts = ignore_groups(input).into_iter();
        let input_str = match (ts.next(), ts.next()) {
            (Some(TokenTree::Literal(literal)), None) => literal.to_string(),
            _ => panic!("expected single string literal"),
        };
        let mut buf: Vec<u8> = input_str.into();

        match buf.as_slice() {
            [b'"', .., b'"'] => (),
            _ => panic!("expected single string literal"),
        };
        buf.pop();
        Self {
            buf,
            pos: 1,
            is_punct: false,
        }
    }

    fn next_hex_val(&mut self) -> Option<u8> {
        loop {
            let v = match self.buf.get(self.pos) {
                Some(&v) => v,
                None => return None,
            };
            self.pos += 1;
            let n = match v {
                b'0'..=b'9' => v - 48,
                b'A'..=b'F' => v - 55,
                b'a'..=b'f' => v - 87,
                b' ' | b'\r' | b'\n' | b'\t' => continue,
                _ => panic!("encountered invalid character"),
            };
            return Some(n);
        }
    }
}

impl Iterator for TokenTreeIter {
    type Item = TokenTree;

    fn next(&mut self) -> Option<TokenTree> {
        let v = if self.is_punct {
            TokenTree::Punct(Punct::new(',', Spacing::Alone))
        } else {
            let p1 = self.next_hex_val()?;
            let p2 = match self.next_hex_val() {
                Some(v) => v,
                None => panic!("expected even number of hex characters"),
            };
            let val = (p1 << 4) + p2;
            TokenTree::Literal(Literal::u8_suffixed(val))
        };
        self.is_punct = !self.is_punct;
        Some(v)
    }
}

/// Macro for converting string literal containing hex-encoded string
/// to an array containing decoded bytes
#[proc_macro]
pub fn hex(input: TokenStream) -> TokenStream {
    let ts = TokenStream::from_iter(TokenTreeIter::new(input));
    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Bracket, ts)))
}
