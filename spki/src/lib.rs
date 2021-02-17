//! [X.509] Subject Public Key Info (SPKI) types describing public keys and their
//! associated [`AlgorithmIdentifier`] OIDs.
//!
//! Described in [RFC 5280 Section 4.1].
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.47** at a minimum.
//!
//! [X.509]: https://en.wikipedia.org/wiki/X.509
//! [RFC 5280 Section 4.1]: https://tools.ietf.org/html/rfc5280#section-4.1

#![no_std]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/spki/0.0.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

mod algorithm;
mod spki;

pub use crate::{
    algorithm::{AlgorithmIdentifier, AlgorithmParameters},
    spki::SubjectPublicKeyInfo,
};
pub use der::{self, ObjectIdentifier};
