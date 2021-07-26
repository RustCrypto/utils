//! Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #1:
//!
//! RSA Cryptography Specifications Version 2.2 ([RFC 8017])
//!
//! ## About
//!
//! This crate supports encoding and decoding RSA private and public keys
//! in either PKCS#1 DER (binary) or PEM (text) formats.
//!
//! PEM encoded RSA private keys begin with:
//!
//! ```text
//! -----BEGIN RSA PRIVATE KEY-----
//! ```
//!
//! PEM encoded RSA public keys begin with:
//!
//! ```text
//! -----BEGIN RSA PUBLIC KEY-----
//! ```
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
    html_root_url = "https://docs.rs/pkcs1/0.2.3"
)]
#![forbid(unsafe_code, clippy::unwrap_used)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod error;
mod private_key;
mod public_key;
mod traits;
mod version;

#[cfg(feature = "alloc")]
mod document;

pub use der::{self, asn1::UIntBytes};

pub use self::{
    error::{Error, Result},
    private_key::RsaPrivateKey,
    public_key::RsaPublicKey,
    traits::{FromRsaPrivateKey, FromRsaPublicKey},
    version::Version,
};

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
pub use pem_rfc7468::LineEnding;

#[cfg(feature = "alloc")]
pub use crate::{
    document::{private_key::RsaPrivateKeyDocument, public_key::RsaPublicKeyDocument},
    traits::{ToRsaPrivateKey, ToRsaPublicKey},
};

#[cfg(feature = "pem")]
use pem_rfc7468 as pem;
