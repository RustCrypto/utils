//! Pure Rust embedded-friendly implementation of the Distinguished Encoding Rules (DER)
//! for Abstract Syntax Notation One (ASN.1) as described in ITU [X.690].
//!
//! # About
//!
//! This crate provides a low-level implementation of a subset of ASN.1 DER
//! necessary for decoding/encoding various cryptography-related formats.
//!
//! It avoids any heap usage, and is presently specialized for documents which
//! are smaller than 64kB.
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.46** at a minimum.
//!
//! [X.690]: https://www.itu.int/rec/T-REC-X.690/

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png",
    html_root_url = "https://docs.rs/der/0.0.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

#[cfg(feature = "std")]
extern crate std;

pub mod decode;
pub mod encode;
mod error;
mod tag;

pub use crate::{
    error::{Error, Result},
    tag::Tag,
};

#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub use oid::ObjectIdentifier;
