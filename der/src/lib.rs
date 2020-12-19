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

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod asn1;
mod byte_slice;
mod decoder;
mod encoder;
mod error;
mod header;
mod length;
mod tag;
mod traits;

pub use crate::{
    asn1::{
        any::Any, bit_string::BitString, boolean::Boolean, integer::Integer, null::Null,
        octet_string::OctetString, sequence::Sequence,
    },
    decoder::Decoder,
    encoder::Encoder,
    error::{Error, Result},
    length::Length,
    tag::Tag,
    traits::{Decodable, Encodable, Message, Tagged},
};

pub(crate) use crate::{byte_slice::ByteSlice, header::Header};

#[cfg(feature = "oid")]
#[cfg_attr(docsrs, doc(cfg(feature = "oid")))]
pub use const_oid::ObjectIdentifier;
