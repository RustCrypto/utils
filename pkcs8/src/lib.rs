//! Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #8:
//! Private-Key Information Syntax Specification (RFC 5208).
//!
//! # About
//!
//! This is a minimalistic library targeting `no_std` platforms and small code
//! size. It avoids the use of any heap-based data structures.
//!
//! Presently only deserialization is supported.
//!
//! # Supported Algorithms
//!
//! This crate is presently specialized for parsing RSA (`rsaEncryption`)
//! and ECC (`id-ecPublicKey`) keys.
//!
//! Encrypted private keys are presently unsupported.
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.46** at a minimum.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/pkcs8/0.0.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod algorithm;
mod asn1;
mod error;
mod private_key_info;
mod spki;

#[cfg(feature = "alloc")]
mod document;

#[cfg(feature = "pem")]
mod pem;

pub use crate::{
    algorithm::AlgorithmIdentifier,
    error::{Error, Result},
    private_key_info::PrivateKeyInfo,
    spki::SubjectPublicKeyInfo,
};
pub use const_oid::ObjectIdentifier;

#[cfg(feature = "alloc")]
pub use crate::document::{PrivateKeyDocument, PublicKeyDocument};

/// Parse an object from a PKCS#8 encoded document.
pub trait FromPkcs8: Sized {
    /// Parse the `private_key` field of a PKCS#8-encoded private key's
    /// `PrivateKeyInfo`.
    fn from_pkcs8_private_key_info(private_key_info: PrivateKeyInfo<'_>) -> Result<Self>;

    /// Deserialize PKCS#8-encoded private key from ASN.1 DER
    /// (binary format).
    fn from_pkcs8_der(bytes: &[u8]) -> Result<Self> {
        Self::from_pkcs8_private_key_info(PrivateKeyInfo::from_der(bytes)?)
    }

    /// Deserialize PKCS#8-encoded private key from PEM.
    ///
    /// Keys in this format begin with the following delimiter:
    ///
    /// ```text
    /// -----BEGIN PRIVATE KEY-----
    /// ```
    #[cfg(feature = "pem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
    fn from_pkcs8_pem(s: &str) -> Result<Self> {
        Self::from_pkcs8_der(PrivateKeyDocument::from_pem(s)?.as_ref())
    }
}
