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
    html_root_url = "https://docs.rs/pkcs8/0.1.1"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod algorithm;
mod asn1;
mod error;
mod private_key_info;
mod spki;
mod traits;

#[cfg(feature = "alloc")]
mod document;

#[cfg(feature = "pem")]
mod pem;

pub use crate::{
    algorithm::AlgorithmIdentifier,
    error::{Error, Result},
    private_key_info::PrivateKeyInfo,
    spki::SubjectPublicKeyInfo,
    traits::{FromPrivateKey, FromPublicKey},
};
pub use const_oid::ObjectIdentifier;

#[cfg(feature = "alloc")]
pub use crate::{
    document::{PrivateKeyDocument, PublicKeyDocument},
    traits::{ToPrivateKey, ToPublicKey},
};
