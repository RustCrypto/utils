//! Pure Rust implementation of PEM Encoding ([RFC 7468]) for PKIX, PKCS, and
//! CMS Structures, a strict subset of the original Privacy-Enhanced Mail encoding
//! intended  specifically for use with cryptographic keys, certificates, and other
//! messages.
//!
//! Provides a `no_std`-friendly, constant-time implementation suitable for use with
//! cryptographic private keys.
//!
//! # About
//!
//! Many cryptography-related document formats, such as certificates (PKIX),
//! private and public keys/keypairs (PKCS), and other cryptographic messages (CMS)
//! provide an ASCII encoding which can be traced back to Privacy-Enhanced Mail
//! (PEM) as defined in [RFC 1421], which look like the following:
//!
//! ```text
//! -----BEGIN PRIVATE KEY-----
//! MC4CAQAwBQYDK2VwBCIEIBftnHPp22SewYmmEoMcX8VwI4IHwaqd+9LFPj/15eqF
//! -----END PRIVATE KEY-----
//! ```
//!
//! However, all of these formats actually implement a text-based encoding that is
//! similar to, but *not* identical with, the legacy PEM encoding as described in
//! [RFC 1421].
//!
//! For this reason, [RFC 7468] was created to describe a stricter form of
//! "PEM encoding" for use in these applications which codifies the previously
//! de facto rules that most implementations operate by, and makes recommendations
//! to promote interoperability.
//!
//! This crate attempts to implement a strict interpretation of the [RFC 7468]
//! rules, implementing all of the MUSTs and SHOULDs while avoiding the MAYs,
//! and targeting the "ABNF (Strict)" subset of the grammar as described in
//! [RFC 7468 Section 3 Figure 3 (p6)][RFC 7468 p6].
//!
//! # Implementation notes
//!
//! - Core PEM implementation is `no_std`-friendly and requires no heap allocations.
//! - Avoids use of copies and temporary buffers.
//! - Uses the [`base64ct`] crate to decode/encode Base64 in constant-time.
//! - PEM parser avoids branching on potentially secret data as much as
//!   possible. In the happy path, only 1-byte of secret data is potentially
//!   branched upon.
//!
//! Note: a forthcoming paper [Util::Lookup: Exploiting key decoding in cryptographic libraries][Util::Lookup]
//! demonstrates how the leakage from non-constant-time PEM parsers can be used
//! to practically extract RSA private keys from SGX enclaves.
//!
//! # Minimum Supported Rust Version
//!
//! This crate requires **Rust 1.51** at a minimum.
//!
//! # Usage
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # #[cfg(feature = "alloc")]
//! # {
//! /// Example PEM document
//! /// NOTE: do not actually put private key literals into your source code!!!
//! let example_pem = "\
//! -----BEGIN PRIVATE KEY-----
//! MC4CAQAwBQYDK2VwBCIEIBftnHPp22SewYmmEoMcX8VwI4IHwaqd+9LFPj/15eqF
//! -----END PRIVATE KEY-----
//! ";
//!
//! // Decode PEM
//! let (type_label, data) = pem_rfc7468::decode_vec(example_pem.as_bytes())?;
//! assert_eq!(type_label, "PRIVATE KEY");
//! assert_eq!(
//!     data,
//!     &[
//!         48, 46, 2, 1, 0, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 23, 237, 156, 115, 233, 219,
//!         100, 158, 193, 137, 166, 18, 131, 28, 95, 197, 112, 35, 130, 7, 193, 170, 157, 251,
//!         210, 197, 62, 63, 245, 229, 234, 133
//!     ]
//! );
//!
//! // Encode PEM
//! use pem_rfc7468::LineEnding;
//! let encoded_pem = pem_rfc7468::encode_string(type_label, LineEnding::default(), &data)?;
//! assert_eq!(&encoded_pem, example_pem);
//! # }
//! # Ok(())
//! # }
//! ```
//!
//! [RFC 1421]: https://datatracker.ietf.org/doc/html/rfc1421
//! [RFC 7468]: https://datatracker.ietf.org/doc/html/rfc7468
//! [RFC 7468 p6]: https://datatracker.ietf.org/doc/html/rfc7468#page-6
//! [Util::Lookup]: https://twitter.com/JanWichelmann/status/1418532480081145857

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_root_url = "https://docs.rs/pem-rfc7468/0.1.1"
)]
#![forbid(unsafe_code, clippy::unwrap_used)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

#[cfg(feature = "alloc")]
#[macro_use]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod decoder;
mod encoder;
mod error;
mod grammar;

pub use crate::{
    decoder::decode,
    encoder::{encode, encoded_len, LineEnding},
    error::{Error, Result},
};

#[cfg(feature = "alloc")]
pub use crate::{decoder::decode_vec, encoder::encode_string};

/// The pre-encapsulation boundary appears before the encapsulated text.
///
/// From RFC 7468 Section 2:
/// > There are exactly five hyphen-minus (also known as dash) characters ("-")
/// > on both ends of the encapsulation boundaries, no more, no less.
const PRE_ENCAPSULATION_BOUNDARY: &[u8] = b"-----BEGIN ";

/// The post-encapsulation boundary appears immediately after the encapsulated text.
const POST_ENCAPSULATION_BOUNDARY: &[u8] = b"-----END ";

/// Delimiter of encapsulation boundaries.
const ENCAPSULATION_BOUNDARY_DELIMITER: &[u8] = b"-----";

/// Width at which Base64 must be wrapped.
///
/// From RFC 7468 Section 2:
///
/// > Generators MUST wrap the base64-encoded lines so that each line
/// > consists of exactly 64 characters except for the final line, which
/// > will encode the remainder of the data (within the 64-character line
/// > boundary), and they MUST NOT emit extraneous whitespace.  Parsers MAY
/// > handle other line sizes.
const BASE64_WRAP_WIDTH: usize = 64;
