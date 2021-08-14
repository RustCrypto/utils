//! Pure Rust implementation of Base64 encoding ([RFC 4648, section 4])
//! with a constant-time `no_std`-friendly implementation.
//!
//! # About
//!
//! This crate implements several Base64 variants in constant-time.
//!
//! The padded variants require (`=`) padding. Unpadded variants expressly
//! reject such padding.
//!
//! Whitespace is expressly disallowed.
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.47** at a minimum.
//!
//! We may change the MSRV in the future, but it will be accompanied by a minor
//! version bump.
//!
//! # Usage
//!
//! ## Allocating (enable `alloc` crate feature)
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use base64ct::{Base64, Encoding};
//!
//! let bytes = b"example bytestring!";
//! let encoded = Base64::encode_string(bytes);
//! assert_eq!(encoded, "ZXhhbXBsZSBieXRlc3RyaW5nIQ==");
//!
//! let decoded = Base64::decode_vec(&encoded).unwrap();
//! assert_eq!(decoded, bytes);
//! # }
//! ```
//!
//! ## Heapless `no_std` usage
//!
//! ```
//! use base64ct::{Base64, Encoding};
//!
//! const BUF_SIZE: usize = 128;
//!
//! let bytes = b"example bytestring!";
//! assert!(Base64::encoded_len(bytes) <= BUF_SIZE);
//!
//! let mut enc_buf = [0u8; BUF_SIZE];
//! let encoded = Base64::encode(bytes, &mut enc_buf).unwrap();
//! assert_eq!(encoded, "ZXhhbXBsZSBieXRlc3RyaW5nIQ==");
//!
//! let mut dec_buf = [0u8; BUF_SIZE];
//! let decoded = Base64::decode(encoded, &mut dec_buf).unwrap();
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
    html_root_url = "https://docs.rs/base64ct/1.0.1"
)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod encoding;
mod errors;
mod variant;

pub use crate::{
    encoding::Encoding,
    errors::{Error, InvalidEncodingError, InvalidLengthError},
    variant::{
        bcrypt::Base64Bcrypt,
        crypt::Base64Crypt,
        standard::{Base64, Base64Unpadded},
        url::{Base64Url, Base64UrlUnpadded},
    },
};
