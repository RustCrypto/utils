//! This crate provides the `hex!` macro for converting hexadecimal string literals
//! to a byte array at compile time.
//!
//! It accepts the following characters in the input string:
//!
//! - `'0'...'9'`, `'a'...'f'`, `'A'...'F'` — hex characters which will be used
//!     in construction of the output byte array
//! - `' '`, `'\r'`, `'\n'`, `'\t'` — formatting characters which will be
//!     ignored
//!
//! Additionally it accepts line (`//`) and block (`/* .. */`) comments. Characters
//! inside of those are ignored.
//!
//! # Examples
//! ```
//! # #[macro_use] extern crate hex_literal;
//! // the macro can be used in const context
//! const DATA: [u8; 4] = hex!("01020304");
//! # fn main() {
//! assert_eq!(DATA, [1, 2, 3, 4]);
//!
//! // it understands both upper and lower hex values
//! assert_eq!(hex!("a1 b2 c3 d4"), [0xA1, 0xB2, 0xC3, 0xD4]);
//! assert_eq!(hex!("E5 E6 90 92"), [0xE5, 0xE6, 0x90, 0x92]);
//! assert_eq!(hex!("0a0B 0C0d"), [10, 11, 12, 13]);
//! let bytes = hex!("
//!     00010203 04050607
//!     08090a0b 0c0d0e0f
//! ");
//! assert_eq!(bytes, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
//!
//! // it's possible to use several literals (results will be concatenated)
//! let bytes2 = hex!(
//!     "00010203 04050607" // first half
//!     "08090a0b 0c0d0e0f" // second hald
//! );
//! assert_eq!(bytes2, bytes);
//!
//! // comments can be also included inside literals
//! assert_eq!(hex!("0a0B // 0c0d line comments"), [10, 11]);
//! assert_eq!(hex!("0a0B // line comments
//!                  0c0d"), [10, 11, 12, 13]);
//! assert_eq!(hex!("0a0B /* block comments */ 0c0d"), [10, 11, 12, 13]);
//! assert_eq!(hex!("0a0B /* multi-line
//!                          block comments
//!                       */ 0c0d"), [10, 11, 12, 13]);
//! # }
//! ```
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/hex-literal/0.3.3"
)]

mod comments;
extern crate proc_macro;

use std::vec::IntoIter;

use proc_macro::{Delimiter, Group, Literal, Punct, Spacing, TokenStream, TokenTree};

use crate::comments::{Exclude, ExcludingComments};

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
    buf: ExcludingComments<IntoIter<u8>>,
    is_punct: bool,
}

impl TokenTreeIter {
    fn new(input: Literal) -> Self {
        let mut buf: Vec<u8> = input.to_string().into();

        match buf.as_slice() {
            [b'"', .., b'"'] => (),
            _ => panic!("expected string literals"),
        };
        buf.pop();
        let mut iter = buf.into_iter().exclude_comments();
        iter.next();
        Self {
            buf: iter,
            is_punct: false,
        }
    }

    fn next_hex_val(&mut self) -> Option<u8> {
        loop {
            let v = self.buf.next()?;
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

/// Macro for converting sequence of string literals containing hex-encoded data
/// into an array of bytes.
#[proc_macro]
pub fn hex(input: TokenStream) -> TokenStream {
    let mut out_ts = TokenStream::new();
    for tt in ignore_groups(input) {
        let iter = match tt {
            TokenTree::Literal(literal) => TokenTreeIter::new(literal),
            _ => panic!("expected string literals"),
        };
        out_ts.extend(iter);
    }
    TokenStream::from(TokenTree::Group(Group::new(Delimiter::Bracket, out_ts)))
}
