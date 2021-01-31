//! Pure Rust implementation of Base64 encoding ([RFC 4648, section 4])
//! with a constant-time `no_std`-friendly implementation.
//!
//! # About
//!
//! This crate implements the following Base64 variants in constant-time:
//!
//! - Standard Base64 encoding: `[A-Za-z0-9+/]`
//!   - [`base64ct::padded`][`padded`]
//!   - [`base64ct::unpadded`][`unpadded`]
//! - URL-safe Base64: `[A-Za-z0-9\-_]`
//!   - [`base64ct::url::padded`][`url::padded`]
//!   - [`base64ct::url::unpadded`][`url::unpadded`]
//!
//! The padded variants require (`=`) padding. Unpadded variants expressly
//! reject such padding.
//!
//! Whitespace is expressly disallowed.
//!
//! # Usage
//!
//! ## Allocating (enable `alloc` crate feature)
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use base64ct::padded as base64;
//!
//! let bytes = b"example bytestring!";
//! let encoded = base64::encode_string(bytes);
//! assert_eq!(encoded, "ZXhhbXBsZSBieXRlc3RyaW5nIQ==");
//!
//! let decoded = base64::decode_vec(&encoded).unwrap();
//! assert_eq!(decoded, bytes);
//! # }
//! ```
//!
//! ## Heapless `no_std` usage
//!
//! ```
//! use base64ct::padded as base64;
//!
//! const BUF_SIZE: usize = 128;
//!
//! let bytes = b"example bytestring!";
//! assert!(base64::encoded_len(bytes) <= BUF_SIZE);
//!
//! let mut enc_buf = [0u8; BUF_SIZE];
//! let encoded = base64::encode(bytes, &mut enc_buf).unwrap();
//! assert_eq!(encoded, "ZXhhbXBsZSBieXRlc3RyaW5nIQ==");
//!
//! let mut dec_buf = [0u8; BUF_SIZE];
//! let decoded = base64::decode(encoded, &mut dec_buf).unwrap();
//! assert_eq!(decoded, bytes);
//! ```
//!
//! # Implementation
//!
//! Implemented using bitwise arithmetic alone without any lookup tables or
//! data-dependent branches, thereby providing portable "best effort"
//! constant-time operation.
//!
//! Not constant-time with respect to message length (only data).
//!
//! Adapted from the following constant-time C++ implementation of Base64:
//!
//! <https://github.com/Sc00bz/ConstTimeEncoding/blob/master/base64.cpp>
//!
//! Copyright (c) 2014 Steve "Sc00bz" Thomas (steve at tobtu dot com).
//! Derived code is dual licensed MIT + Apache 2 (with permission from Sc00bz).
//!
//! [RFC 4648, section 4]: https://tools.ietf.org/html/rfc4648#section-4

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/base64ct/0.1.1"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod url;

mod decoder;
mod encoder;
mod errors;
mod standard;

pub use errors::{Error, InvalidEncodingError, InvalidLengthError};
pub use standard::{padded, unpadded};

/// Padding character
const PAD: u8 = b'=';
