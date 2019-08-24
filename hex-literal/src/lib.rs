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
#![doc(html_logo_url =
    "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]
#![no_std]

use proc_macro_hack::proc_macro_hack;
#[doc(hidden)]
#[proc_macro_hack]
pub use hex_literal_impl::hex;
