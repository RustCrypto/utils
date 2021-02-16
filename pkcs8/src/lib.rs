//! Pure Rust implementation of Public-Key Cryptography Standards (PKCS) #8:
//! Private-Key Information Syntax Specification (as defined in [RFC 5208]).
//!
//! # About
//!
//! This is a minimalistic library targeting `no_std` platforms and small code
//! size. It supports decoding/encoding of the following types without the use
//! of a heap:
//!
//! - [`PrivateKeyInfo`]: algorithm identifier and data representing a private key.
//! - [`SubjectPublicKeyInfo`]: algorithm identifier and data representing a public key
//!   (re-exported from the [`spki`] crate)
//!
//! When the `alloc` feature is enabled, the following additional types are
//! available which provide more convenient decoding/encoding support:
//!
//! - [`PrivateKeyDocument`]: heap-backed storage for serialized [`PrivateKeyInfo`].
//! - [`PublicKeyDocument`]: heap-backed storage for serialized [`SubjectPublicKeyInfo`].
//!
//! When the `pem` feature is enabled, it also supports decoding/encoding
//! documents from "PEM encoding" format as defined in RFC 7468.
//!
//! # Supported Algorithms
//!
//! This crate has been tested against keys generated by OpenSSL for the
//! following algorithms:
//!
//! - ECC (`id-ecPublicKey`)
//! - Ed25519 (`Ed25519`)
//! - RSA (`rsaEncryption`)
//!
//! It may work with other algorithms which use an optional OID for
//! [`AlgorithmIdentifier`] parameters.
//!
//! Encrypted private keys are presently unsupported.
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.47** at a minimum.
//!
//! [RFC 5208]: https://tools.ietf.org/html/rfc5208

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/pkcs8/0.5.0-pre"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod error;
mod private_key_info;
mod traits;

#[cfg(feature = "alloc")]
mod document;

#[cfg(feature = "pem")]
mod pem;

pub use crate::{
    error::{Error, Result},
    private_key_info::PrivateKeyInfo,
    traits::{FromPrivateKey, FromPublicKey},
};
pub use der::{self, ObjectIdentifier};
pub use spki::{AlgorithmIdentifier, AlgorithmParameters, SubjectPublicKeyInfo};

#[cfg(feature = "alloc")]
pub use crate::{
    document::{PrivateKeyDocument, PublicKeyDocument},
    traits::{ToPrivateKey, ToPublicKey},
};
