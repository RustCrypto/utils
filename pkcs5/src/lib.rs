//! Pure Rust implementation of  Public-Key Cryptography Standards (PKCS) #5:
//! Password-Based Cryptography Specification Version 2.1 ([RFC 8018])
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.47** at a minimum.
//!
//! [RFC 8018]: https://tools.ietf.org/html/rfc8018

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/pkcs5/0.0.0"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

pub use der::{self, Error, ObjectIdentifier, Result};
pub use spki::AlgorithmIdentifier;

use core::convert::{TryFrom, TryInto};
use der::{sequence, Any, Encodable, Encoder, Length};

pub mod pbes1;
pub mod pbes2;

/// Supported PKCS#5 password-based encryption schemes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum Scheme<'a> {
    /// Password-Based Encryption Scheme 1 as defined in [RFC 8018 Section 6.1].
    ///
    /// [RFC 8018 Section 6.1]: https://tools.ietf.org/html/rfc8018#section-6.1
    Pbes1(pbes1::Parameters),

    /// Password-Based Encryption Scheme 2 as defined in [RFC 8018 Section 6.2].
    ///
    /// [RFC 8018 Section 6.2]: https://tools.ietf.org/html/rfc8018#section-6.2
    Pbes2(pbes2::Parameters<'a>),
}

impl<'a> Scheme<'a> {
    /// Get the [`ObjectIdentifier`] (a.k.a OID) for this algorithm.
    pub fn oid(&self) -> ObjectIdentifier {
        match self {
            Self::Pbes1(params) => params.oid(),
            Self::Pbes2(_) => pbes2::PBES2_OID,
        }
    }

    /// Get [`pbes1::Parameters`] if it is the selected algorithm.
    pub fn pbes1(&self) -> Option<&pbes1::Parameters> {
        match self {
            Self::Pbes1(params) => Some(params),
            _ => None,
        }
    }

    /// Get [`pbes2::Parameters`] if it is the selected algorithm.
    pub fn pbes2(&self) -> Option<&pbes2::Parameters<'a>> {
        match self {
            Self::Pbes2(params) => Some(params),
            _ => None,
        }
    }
}

impl<'a> TryFrom<&'a [u8]> for Scheme<'a> {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<Scheme<'a>> {
        AlgorithmIdentifier::try_from(bytes).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<Any<'a>> for Scheme<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Scheme<'a>> {
        AlgorithmIdentifier::try_from(any).and_then(TryInto::try_into)
    }
}

impl<'a> TryFrom<AlgorithmIdentifier<'a>> for Scheme<'a> {
    type Error = Error;

    fn try_from(alg: AlgorithmIdentifier<'a>) -> Result<Scheme<'_>> {
        match alg.oid {
            pbes2::PBES2_OID => pbes2::Parameters::try_from(alg.parameters_any()?).map(Into::into),
            _ => pbes1::Parameters::try_from(alg).map(Into::into),
        }
    }
}

impl<'a> From<pbes1::Parameters> for Scheme<'a> {
    fn from(params: pbes1::Parameters) -> Scheme<'a> {
        Self::Pbes1(params)
    }
}

impl<'a> From<pbes2::Parameters<'a>> for Scheme<'a> {
    fn from(params: pbes2::Parameters<'a>) -> Scheme<'a> {
        Self::Pbes2(params)
    }
}

impl<'a> Encodable for Scheme<'a> {
    fn encoded_len(&self) -> Result<Length> {
        match self {
            Self::Pbes1(pbes1) => pbes1.encoded_len(),
            Self::Pbes2(pbes2) => sequence::encoded_len(&[&pbes2::PBES2_OID, pbes2]),
        }
    }

    fn encode(&self, encoder: &mut Encoder<'_>) -> Result<()> {
        match self {
            Self::Pbes1(pbes1) => pbes1.encode(encoder),
            Self::Pbes2(pbes2) => encoder.sequence(&[&pbes2::PBES2_OID, pbes2]),
        }
    }
}
