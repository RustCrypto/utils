//! Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #1:
//! RSA Cryptography Specifications Version 2.2 ([RFC 8017])
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.51** at a minimum.
//!
//! [RFC 8017]: https://tools.ietf.org/html/rfc8017
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/pkcs1/0.0.0"
)]
#![forbid(unsafe_code, clippy::unwrap_used)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(feature = "std")]
extern crate std;

mod error;
mod private_key;
mod public_key;
mod version;

pub use self::{
    error::{Error, Result},
    private_key::RsaPrivateKey,
    public_key::RsaPublicKey,
    version::Version,
};
