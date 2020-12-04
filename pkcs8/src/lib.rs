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
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/pkcs8/0.0.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "std")]
extern crate std;

mod algorithm;
mod asn1;
mod error;

pub use crate::{
    algorithm::AlgorithmIdentifier,
    error::{Error, Result},
};
pub use const_oid::ObjectIdentifier;

/// PKCS#8 `PrivateKeyInfo`.
///
/// ASN.1 structure containing an [`AlgorithmIdentifier`] and private key
/// data in an algorithm specific format.
///
/// Described in RFC 5208 Section 5:
/// <https://tools.ietf.org/html/rfc5208#section-5>
///
/// ```text
/// PrivateKeyInfo ::= SEQUENCE {
///         version                   Version,
///         privateKeyAlgorithm       PrivateKeyAlgorithmIdentifier,
///         privateKey                PrivateKey,
///         attributes           [0]  IMPLICIT Attributes OPTIONAL }
///
/// Version ::= INTEGER
///
/// PrivateKeyAlgorithmIdentifier ::= AlgorithmIdentifier
///
/// PrivateKey ::= OCTET STRING
///
/// Attributes ::= SET OF Attribute
/// ```
pub struct PrivateKeyInfo<'a> {
    /// X.509 [`AlgorithmIdentifier`]
    pub algorithm: AlgorithmIdentifier,

    /// Private key data
    pub private_key: &'a [u8],
}

impl<'a> PrivateKeyInfo<'a> {
    /// Parse [`PrivateKeyInfo`] encoded as ASN.1 DER
    pub fn from_der(bytes: &'a [u8]) -> Result<Self> {
        asn1::parse_private_key_info(bytes)
    }
}
